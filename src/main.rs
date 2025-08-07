use std::env;
use std::ffi::OsString;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::ptr;
use std::thread;
use std::time::Duration;
use winapi::shared::minwindef::{DWORD, FALSE, HKEY, TRUE};
use winapi::shared::ntdef::HANDLE;
use winapi::shared::winerror::ERROR_MORE_DATA;
use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
use winapi::um::processthreadsapi::{GetCurrentProcess, OpenProcess, TerminateProcess};
use winapi::um::psapi::{EnumProcesses, GetModuleBaseNameW, GetProcessImageFileNameW};
use winapi::um::restartmanager::*;
use winapi::um::winnt::{KEY_WRITE, REG_SZ};
use winapi::um::winnt::{PROCESS_QUERY_INFORMATION, PROCESS_TERMINATE, PROCESS_VM_READ};
use winapi::um::winreg::{HKEY_CLASSES_ROOT, RegCloseKey, RegCreateKeyExW, RegSetValueExW};

#[derive(Debug, Clone)]
struct ProcessInfo {
    pid: u32,
    name: String,
    path: String,
}

#[derive(Debug)]
struct FileLockInfo {
    file_path: String,
    processes: Vec<ProcessInfo>,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage(&args[0]);
        return;
    }

    let command = &args[1];

    match command.as_str() {
        "install" => match install_context_menu() {
            Ok(_) => println!("Context menu installed successfully!"),
            Err(e) => eprintln!("Failed to install context menu: {}", e),
        },
        "uninstall" => match uninstall_context_menu() {
            Ok(_) => println!("Context menu uninstalled successfully!"),
            Err(e) => eprintln!("Failed to uninstall context menu: {}", e),
        },
        "check" => {
            if args.len() != 3 {
                println!("Usage: {} check <file_path>", args[0]);
                return;
            }
            check_file_command(&args[2]);
        }
        "unlock" => {
            if args.len() != 3 {
                println!("Usage: {} unlock <file_path>", args[0]);
                return;
            }
            unlock_file_command(&args[2]);
        }
        "monitor" => {
            if args.len() < 3 {
                println!("Usage: {} monitor <file_path> [interval_seconds]", args[0]);
                return;
            }
            let interval = if args.len() >= 4 {
                args[3].parse().unwrap_or(2)
            } else {
                2
            };
            monitor_file_command(&args[2], interval);
        }
        _ => {
            print_usage(&args[0]);
        }
    }
}

fn print_usage(program_name: &str) {
    println!("File Lock Checker v1.0");
    println!("Usage:");
    println!(
        "  {} install   - Install context menu entries (Note: Must run as Administrator!)",
        program_name
    );
    println!("  {} uninstall - Remove context menu entries (Note: Must run as Administrator!)", program_name);
    println!(
        "  {} check <file_path>                    - Check which processes are locking the file",
        program_name
    );
    println!(
        "  {} unlock <file_path>                   - Kill processes that are locking the file",
        program_name
    );
    println!(
        "  {} monitor <file_path> [interval]       - Monitor file locks in real-time (default: 2 seconds)",
        program_name
    );
    println!();
    println!("Examples:");
    println!("  {} install", program_name);
    println!("  {} uninstall", program_name);
    println!("  {} check \"C:\\temp\\locked_file.txt\"", program_name);
    println!("  {} unlock \"C:\\temp\\locked_file.txt\"", program_name);
    println!("  {} monitor \"C:\\temp\\locked_file.txt\" 5", program_name);
}

// ติดตั้ง context menu ทั้งหมด
fn install_context_menu() -> Result<(), String> {
    // หา path ของ executable ปัจจุบัน
    let exe_path =
        env::current_exe().map_err(|e| format!("Failed to get executable path: {}", e))?;

    let exe_path_str = exe_path.to_str().ok_or("Invalid executable path")?;

    println!("Installing context menu for: {}", exe_path_str);

    // ติดตั้งสำหรับ All Files (*)
    install_for_file_type("*", exe_path_str)?;

    // ติดตั้งสำหรับ Folders
    install_for_folders(exe_path_str)?;

    println!("Context menu installed for all file types and folders");
    Ok(())
}

// ติดตั้งสำหรับไฟล์ประเภทหนึ่ง
fn install_for_file_type(file_type: &str, exe_path: &str) -> Result<(), String> {
    let base_key = format!("{}\\shell", file_type);

    // สร้าง menu items
    create_menu_item(&base_key, "FLC_Check", "FLC Check", exe_path, "check")?;
    create_menu_item(&base_key, "FLC_Unlock", "FLC Unlock", exe_path, "unlock")?;
    create_menu_item(&base_key, "FLC_Monitor", "FLC Monitor", exe_path, "monitor")?;

    Ok(())
}

// ติดตั้งสำหรับ folders
fn install_for_folders(exe_path: &str) -> Result<(), String> {
    let base_key = "Directory\\shell";

    // สร้าง menu items สำหรับ folder
    create_menu_item(base_key, "FLC_Check", "FLC Check", exe_path, "check")?;
    create_menu_item(base_key, "FLC_Monitor", "FLC Monitor", exe_path, "monitor")?;

    Ok(())
}

// สร้าง menu item เดียว
fn create_menu_item(
    base_key: &str,
    key_name: &str,
    display_name: &str,
    exe_path: &str,
    command: &str,
) -> Result<(), String> {
    unsafe {
        // สร้าง key หลัก
        let menu_key_path = format!("{}\\{}", base_key, key_name);
        let mut menu_key: HKEY = ptr::null_mut();

        let wide_menu_key: Vec<u16> = OsString::from(&menu_key_path)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let result = RegCreateKeyExW(
            HKEY_CLASSES_ROOT,
            wide_menu_key.as_ptr(),
            0,
            ptr::null_mut(),
            0,
            KEY_WRITE,
            ptr::null_mut(),
            &mut menu_key,
            ptr::null_mut(),
        );

        if result != 0 {
            return Err(format!("Failed to create registry key: {}", result));
        }

        // ตั้งค่า display name
        set_registry_value(menu_key, "", display_name)?;

        // ตั้งค่า icon (ใช้ icon ของ exe)
        let icon_value = format!("{},0", exe_path);
        set_registry_value(menu_key, "Icon", &icon_value)?;

        RegCloseKey(menu_key);

        // สร้าง command subkey
        let command_key_path = format!("{}\\command", menu_key_path);
        let mut command_key: HKEY = ptr::null_mut();

        let wide_command_key: Vec<u16> = OsString::from(&command_key_path)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let result = RegCreateKeyExW(
            HKEY_CLASSES_ROOT,
            wide_command_key.as_ptr(),
            0,
            ptr::null_mut(),
            0,
            KEY_WRITE,
            ptr::null_mut(),
            &mut command_key,
            ptr::null_mut(),
        );

        if result != 0 {
            return Err(format!("Failed to create command key: {}", result));
        }

        // ตั้งค่า command
        let command_value = format!("\"{}\" {} \"%1\"", exe_path, command);
        set_registry_value(command_key, "", &command_value)?;

        RegCloseKey(command_key);
    }

    Ok(())
}

// ตั้งค่า registry value
fn set_registry_value(key: HKEY, name: &str, value: &str) -> Result<(), String> {
    unsafe {
        let wide_name: Vec<u16> = if name.is_empty() {
            vec![0]
        } else {
            OsString::from(name)
                .encode_wide()
                .chain(std::iter::once(0))
                .collect()
        };

        let wide_value: Vec<u16> = OsString::from(value)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let result = RegSetValueExW(
            key,
            wide_name.as_ptr(),
            0,
            REG_SZ,
            wide_value.as_ptr() as *const u8,
            (wide_value.len() * 2) as DWORD,
        );

        if result != 0 {
            return Err(format!("Failed to set registry value: {}", result));
        }
    }

    Ok(())
}

// ถอนการติดตั้ง context menu
fn uninstall_context_menu() -> Result<(), String> {
    unsafe {
        // ลบ menu items สำหรับ All Files
        delete_registry_key("*\\shell\\FLC_Check")?;
        delete_registry_key("*\\shell\\FLC_Unlock")?;
        delete_registry_key("*\\shell\\FLC_Monitor")?;

        // ลบ menu items สำหรับ Folders
        delete_registry_key("Directory\\shell\\FLC_Check")?;
        delete_registry_key("Directory\\shell\\FLC_Monitor")?;
    }

    Ok(())
}

// ลบ registry key
fn delete_registry_key(key_path: &str) -> Result<(), String> {
    unsafe {
        use winapi::um::winreg::RegDeleteTreeW;

        let wide_key: Vec<u16> = OsString::from(key_path)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let result = RegDeleteTreeW(HKEY_CLASSES_ROOT, wide_key.as_ptr());

        // ไม่ต้องแสดง error ถ้า key ไม่มีอยู่แล้ว
        if result != 0 && result != 2 {
            // ERROR_FILE_NOT_FOUND = 2
            eprintln!("Warning: Failed to delete key {}: {}", key_path, result);
        }
    }

    Ok(())
}

// คำสั่งตรวจสอบไฟล์ที่ถูก lock
fn check_file_command(file_path: &str) {
    match check_file_locks(file_path) {
        Ok(lock_info) => {
            print_lock_info(&lock_info);
        }
        Err(e) => {
            eprintln!("Error checking file locks: {}", e);
        }
    }
    print!("Press enter to continue...");
    io::stdout().flush().unwrap();
    let mut _wait = String::new();
    let _ = io::stdin().read_line(&mut _wait);
}

// คำสั่ง unlock ไฟล์โดยการ kill processes
fn unlock_file_command(file_path: &str) {
    match check_file_locks(file_path) {
        Ok(lock_info) => {
            if lock_info.processes.is_empty() {
                println!("File '{}' is not locked by any process", file_path);
                return;
            }

            print_lock_info(&lock_info);
            println!();

            print!("Do you want to kill these processes? (y/N): ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();

            if input.trim().to_lowercase() == "y" || input.trim().to_lowercase() == "yes" {
                kill_processes(&lock_info.processes);

                // ตรวจสอบอีกครั้งหลังจาก kill processes
                println!("\nRechecking file locks...");
                thread::sleep(Duration::from_millis(500));

                match check_file_locks(file_path) {
                    Ok(new_lock_info) => {
                        if new_lock_info.processes.is_empty() {
                            println!("File '{}' is now unlocked", file_path);
                        } else {
                            println!("Warning: Some processes are still locking the file:");
                            print_lock_info(&new_lock_info);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error rechecking file locks: {}", e);
                    }
                }
            } else {
                println!("Operation cancelled");
            }
        }
        Err(e) => {
            eprintln!("Error checking file locks: {}", e);
        }
    }
}

// คำสั่ง monitor ไฟล์แบบ real-time
fn monitor_file_command(file_path: &str, interval_seconds: u64) {
    println!("Monitoring file locks for: {}", file_path);
    println!("Update interval: {} seconds", interval_seconds);
    println!("Press Ctrl+C to stop monitoring\n");

    let mut last_processes: Vec<ProcessInfo> = Vec::new();

    loop {
        match check_file_locks(file_path) {
            Ok(lock_info) => {
                // ตรวจสอบว่ามีการเปลี่ยนแปลงหรือไม่
                if processes_changed(&last_processes, &lock_info.processes) {
                    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
                    println!("[{}] Lock status changed:", timestamp);

                    if lock_info.processes.is_empty() {
                        println!("  File is now UNLOCKED");
                    } else {
                        println!(
                            "  File is LOCKED by {} process(es):",
                            lock_info.processes.len()
                        );
                        for process in &lock_info.processes {
                            println!(
                                "    PID: {} | Name: {} | Path: {}",
                                process.pid, process.name, process.path
                            );
                        }
                    }
                    println!();

                    last_processes = lock_info.processes;
                }
            }
            Err(e) => {
                eprintln!("Error checking file locks: {}", e);
            }
        }

        thread::sleep(Duration::from_secs(interval_seconds));
    }
}

// ตรวจสอบว่าไฟล์ถูก lock หรือไม่ และโดย process ใด
// หมายเหตุ: Restart Manager API อาจไม่ครอบคลุม kernel-level locks และ system services
fn check_file_locks(file_path: &str) -> Result<FileLockInfo, String> {
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

// แปลง wide string (UTF-16) เป็น Rust String
fn wide_string_to_string(wide_str: &[u16]) -> String {
    let null_pos = wide_str
        .iter()
        .position(|&c| c == 0)
        .unwrap_or(wide_str.len());
    OsString::from_wide(&wide_str[..null_pos])
        .to_string_lossy()
        .into_owned()
}

// ดึง path ของ process จาก PID
fn get_process_path(pid: u32) -> String {
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

// แสดงข้อมูล lock ของไฟล์
fn print_lock_info(lock_info: &FileLockInfo) {
    if lock_info.processes.is_empty() {
        println!(
            "File '{}' is not locked by any process",
            lock_info.file_path
        );

        // ตรวจสอบเพิ่มเติมสำหรับไฟล์ .sys
        if lock_info.file_path.ends_with(".sys") {
            println!("Note: .sys files (kernel drivers) may be locked by kernel-level processes");
            println!("      that are not detected by Restart Manager API.");
            println!("      Try using 'handle.exe' from Sysinternals for more detailed analysis.");
        }

        // ทดสอบการเขียนไฟล์
        if let Err(_) = test_file_access(&lock_info.file_path) {
            println!("Warning: File appears to be locked despite no processes found.");
            println!("         This may indicate kernel-level or system service locks.");
        }
    } else {
        println!(
            "File '{}' is locked by {} process(es):",
            lock_info.file_path,
            lock_info.processes.len()
        );
        println!("{:-<80}", "");
        for (i, process) in lock_info.processes.iter().enumerate() {
            println!("Process #{}", i + 1);
            println!("  PID: {}", process.pid);
            println!("  Name: {}", process.name);
            println!("  Path: {}", process.path);
            println!();
        }
    }
}

// kill processes ทั้งหมดที่ lock ไฟล์
fn kill_processes(processes: &[ProcessInfo]) {
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
fn kill_process(pid: u32) -> Result<(), String> {
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
fn processes_changed(old_processes: &[ProcessInfo], new_processes: &[ProcessInfo]) -> bool {
    if old_processes.len() != new_processes.len() {
        return true;
    }

    // เปรียบเทียบ PID ของ processes
    let old_pids: std::collections::HashSet<u32> = old_processes.iter().map(|p| p.pid).collect();
    let new_pids: std::collections::HashSet<u32> = new_processes.iter().map(|p| p.pid).collect();

    old_pids != new_pids
}

// ทดสอบการเข้าถึงไฟล์โดยตรง
fn test_file_access(file_path: &str) -> Result<(), std::io::Error> {
    // ลองเปิดไฟล์ในโหมด read-write เพื่อทดสอบว่า lock หรือไม่
    OpenOptions::new().read(true).write(true).open(file_path)?;

    Ok(())
}
