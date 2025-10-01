// Copyright (c) 2025 Phumin Maliwan
// SPDX-License-Identifier: MIT

use std::ffi::OsString;
use std::fs::OpenOptions;
use std::os::windows::ffi::OsStringExt;

// แปลง wide string (UTF-16) เป็น Rust String
pub fn wide_string_to_string(wide_str: &[u16]) -> String {
    let null_pos = wide_str
        .iter()
        .position(|&c| c == 0)
        .unwrap_or(wide_str.len());
    OsString::from_wide(&wide_str[..null_pos])
        .to_string_lossy()
        .into_owned()
}


// ทดสอบการเข้าถึงไฟล์โดยตรง
pub fn test_file_access(file_path: &str) -> Result<(), std::io::Error> {
    // ลองเปิดไฟล์ในโหมด read-write เพื่อทดสอบว่า lock หรือไม่
    OpenOptions::new().read(true).write(true).open(file_path)?;

    Ok(())
}