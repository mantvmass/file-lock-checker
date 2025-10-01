// Copyright (c) 2025 Phumin Maliwan
// SPDX-License-Identifier: MIT

use std::ffi::OsString;
use std::os::windows::ffi::OsStrExt;
use std::ptr;
use winapi::shared::minwindef::{DWORD, HKEY};
use winapi::um::winnt::{KEY_WRITE, REG_SZ};
use winapi::um::winreg::{
    RegCloseKey, RegCreateKeyExW, RegDeleteTreeW, RegSetValueExW, HKEY_CLASSES_ROOT,
};

// สร้าง registry
pub fn create_registry_key(
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

// ลบ registry key
pub fn delete_registry_key(key_path: &str) -> Result<(), String> {
    unsafe {
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