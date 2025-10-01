// Copyright (c) 2025 Phumin Maliwan
// SPDX-License-Identifier: MIT

use crate::types::{FileLockInfo, ProcessInfo};
use crate::utils::wide_string_to_string;
use std::ffi::OsString;
use std::os::windows::ffi::OsStrExt;
use std::ptr;
use winapi::shared::minwindef::{DWORD, FALSE};
use winapi::shared::winerror::ERROR_MORE_DATA;
use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
use winapi::um::processthreadsapi::{OpenProcess, TerminateProcess};
use winapi::um::psapi::GetProcessImageFileNameW;
use winapi::um::restartmanager::*;
use winapi::um::winnt::{PROCESS_QUERY_INFORMATION, PROCESS_TERMINATE, PROCESS_VM_READ};

// ตรวจสอบว่าไฟล์ถูก lock หรือไม่ และโดย process ใด
// หมายเหตุ: Restart Manager API อาจไม่ครอบคลุม kernel-level locks และ system services
pub fn check_file_locks(file_path: &str) -> Result<FileLockInfo, String> {
    unsafe {
        let mut session_handle: DWORD = 0;
        let mut session_key = [0u16; CCH_RM_SESSION_KEY + 1];

        // เริ่มต้น Restart Manager session
        let result = RmStartSession(&mut session_handle, 0, session_key.as_mut_ptr());
        if result != 0 {
            return Err(format!(
                "Failed to start Restart Manager session. Error: {}",
                result
            ));
        }

        // แปลง file path เป็น wide string สำหรับ Windows API
        let wide_path: Vec<u16> = OsString::from(file_path)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        // ลงทะเบียน resource (ไฟล์) ที่ต้องการตรวจสอบ
        let mut files = [wide_path.as_ptr()];
        let result = RmRegisterResources(
            session_handle,
            1,                  // จำนวนไฟล์
            files.as_mut_ptr(), // array ของไฟล์
            0,                  // จำนวน services
            ptr::null_mut(),    // array ของ services
            0,                  // จำนวน processes
            ptr::null_mut(),    // array ของ processes
        );

        if result != 0 {
            RmEndSession(session_handle);
            return Err(format!("Failed to register resource. Error: {}", result));
        }

        // ดึงรายการ applications ที่ใช้ resource นี้
        let mut proc_info_needed: DWORD = 0;
        let mut proc_info_count: DWORD = 0;
        let mut reboot_reason: DWORD = 0;

        // เรียกครั้งแรกเพื่อดูว่าต้องการ buffer ขนาดเท่าไร
        let result = RmGetList(
            session_handle,
            &mut proc_info_needed,
            &mut proc_info_count,
            ptr::null_mut(),
            &mut reboot_reason,
        );

        let mut processes = Vec::new();

        // ถ้ามี data มากกว่า buffer ที่เตรียมไว้
        if result == ERROR_MORE_DATA && proc_info_needed > 0 {
            // จัดสรร buffer และดึง data จริง
            let mut proc_info: Vec<RM_PROCESS_INFO> =
                vec![std::mem::zeroed(); proc_info_needed as usize];
            proc_info_count = proc_info_needed;

            let result = RmGetList(
                session_handle,
                &mut proc_info_needed,
                &mut proc_info_count,
                proc_info.as_mut_ptr(),
                &mut reboot_reason,
            );

            if result == 0 {
                // แปลงข้อมูล process เป็น struct ที่เราใช้
                for i in 0..proc_info_count {
                    let proc = &proc_info[i as usize];
                    let process_name = wide_string_to_string(&proc.strAppName);
                    let process_path = get_process_path(proc.Process.dwProcessId);

                    processes.push(ProcessInfo {
                        pid: proc.Process.dwProcessId,
                        name: process_name,
                        path: process_path,
                    });
                }
            }
        }

        // ปิด Restart Manager session
        RmEndSession(session_handle);

        Ok(FileLockInfo {
            file_path: file_path.to_string(),
            processes,
        })
    }
}

// ดึง path ของ process จาก PID
pub fn get_process_path(pid: u32) -> String {
    unsafe {
        // เปิด handle ของ process
        let handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, FALSE, pid);
        if handle == ptr::null_mut() || handle == INVALID_HANDLE_VALUE {
            return format!("Unknown (PID: {})", pid);
        }

        // ดึง path ของ executable
        let mut buffer = [0u16; 1024];
        let result = GetProcessImageFileNameW(handle, buffer.as_mut_ptr(), buffer.len() as u32);

        CloseHandle(handle);

        if result > 0 {
            wide_string_to_string(&buffer[..result as usize])
        } else {
            format!("Unknown (PID: {})", pid)
        }
    }
}

// kill processes ทั้งหมดที่ lock ไฟล์
pub fn kill_processes(processes: &[ProcessInfo]) {
    for process in processes {
        match kill_process(process.pid) {
            Ok(_) => {
                println!(
                    "Successfully killed process: {} (PID: {})",
                    process.name, process.pid
                );
            }
            Err(e) => {
                eprintln!(
                    "Failed to kill process {} (PID: {}): {}",
                    process.name, process.pid, e
                );
            }
        }
    }
}

// kill process เดียวจาก PID
pub fn kill_process(pid: u32) -> Result<(), String> {
    unsafe {
        // เปิด handle ของ process พร้อม permission ในการ terminate
        let handle = OpenProcess(PROCESS_TERMINATE, FALSE, pid);
        if handle == ptr::null_mut() || handle == INVALID_HANDLE_VALUE {
            return Err("Failed to open process handle".to_string());
        }

        // ทำการ terminate process
        let result = TerminateProcess(handle, 1);
        CloseHandle(handle);

        if result == 0 {
            Err("Failed to terminate process".to_string())
        } else {
            Ok(())
        }
    }
}

// ตรวจสอบว่า list ของ processes เปลี่ยนแปลงหรือไม่
pub fn processes_changed(old_processes: &[ProcessInfo], new_processes: &[ProcessInfo]) -> bool {
    if old_processes.len() != new_processes.len() {
        return true;
    }

    // เปรียบเทียบ PID ของ processes
    let old_pids: std::collections::HashSet<u32> = old_processes.iter().map(|p| p.pid).collect();
    let new_pids: std::collections::HashSet<u32> = new_processes.iter().map(|p| p.pid).collect();

    old_pids != new_pids
}