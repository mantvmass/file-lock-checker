// Copyright (c) 2025 Phumin Maliwan
// SPDX-License-Identifier: MIT

use crate::registry::{create_registry_key, delete_registry_key};
use std::env;

// ติดตั้ง context menu ทั้งหมด
pub fn install() -> Result<(), String> {
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
    create_registry_key(&base_key, "FLC_Check", "FLC Check", exe_path, "check")?;
    create_registry_key(&base_key, "FLC_Unlock", "FLC Unlock", exe_path, "unlock")?;
    create_registry_key(&base_key, "FLC_Monitor", "FLC Monitor", exe_path, "monitor")?;

    Ok(())
}

// ติดตั้งสำหรับ folders
fn install_for_folders(exe_path: &str) -> Result<(), String> {
    let base_key = "Directory\\shell";

    // สร้าง menu items สำหรับ folder
    create_registry_key(base_key, "FLC_Check", "FLC Check", exe_path, "check")?;
    create_registry_key(base_key, "FLC_Monitor", "FLC Monitor", exe_path, "monitor")?;

    Ok(())
}

// ถอนการติดตั้ง context menu
pub fn uninstall() -> Result<(), String> {
    // ลบ menu items สำหรับ All Files
    delete_registry_key("*\\shell\\FLC_Check")?;
    delete_registry_key("*\\shell\\FLC_Unlock")?;
    delete_registry_key("*\\shell\\FLC_Monitor")?;

    // ลบ menu items สำหรับ Folders
    delete_registry_key("Directory\\shell\\FLC_Check")?;
    delete_registry_key("Directory\\shell\\FLC_Monitor")?;

    Ok(())
}