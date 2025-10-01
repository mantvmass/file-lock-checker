// Copyright (c) 2025 Phumin Maliwan
// SPDX-License-Identifier: MIT

mod commands;
mod context_menu;
mod process;
mod registry;
mod types;
mod utils;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage(&args[0]);
        return;
    }

    let command = &args[1];

    match command.as_str() {
        "install" => match context_menu::install() {
            Ok(_) => println!("Context menu installed successfully!"),
            Err(e) => eprintln!("Failed to install context menu: {}", e),
        },
        "uninstall" => match context_menu::uninstall() {
            Ok(_) => println!("Context menu uninstalled successfully!"),
            Err(e) => eprintln!("Failed to uninstall context menu: {}", e),
        },
        "check" => {
            if args.len() != 3 {
                println!("Usage: {} check <file_path>", args[0]);
                return;
            }
            commands::check(&args[2]);
        }
        "unlock" => {
            if args.len() != 3 {
                println!("Usage: {} unlock <file_path>", args[0]);
                return;
            }
            commands::unlock(&args[2]);
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
            commands::monitor(&args[2], interval);
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