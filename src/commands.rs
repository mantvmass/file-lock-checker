// Copyright (c) 2025 Phumin Maliwan
// SPDX-License-Identifier: MIT

use crate::process::{check_file_locks, kill_processes, processes_changed};
use crate::types::{FileLockInfo, ProcessInfo};
use crate::utils::test_file_access;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

// คำสั่งตรวจสอบไฟล์ที่ถูก lock
pub fn check(file_path: &str) {
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
pub fn unlock(file_path: &str) {
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
pub fn monitor(file_path: &str, interval_seconds: u64) {
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
