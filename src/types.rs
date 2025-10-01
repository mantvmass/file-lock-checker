// Copyright (c) 2025 Phumin Maliwan
// SPDX-License-Identifier: MIT

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub path: String,
}

#[derive(Debug)]
pub struct FileLockInfo {
    pub file_path: String,
    pub processes: Vec<ProcessInfo>,
}