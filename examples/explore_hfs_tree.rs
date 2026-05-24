// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Recursively explore HFS directory tree

use std::fs::File;
use newton_devices::storage::hfs::HfsVolume;
use std::collections::VecDeque;

fn main() {
    let file = File::open("disks/macos-922-uni.iso").expect("Failed to open ISO");
    let hfs_offset = 329 * 512;
    
    println!("Opening HFS volume...");
    let mut volume = HfsVolume::open(file, hfs_offset).expect("Failed to open HFS volume");
    println!("Volume: {}\n", volume.mdb.volume_name);
    
    // BFS traversal of the directory tree
    let mut queue = VecDeque::new();
    queue.push_back((2u32, String::from(""), 0usize)); // Root directory ID=2
    
    let mut total_files = 0;
    let mut total_dirs = 0;
    
    while let Some((dir_id, path, depth)) = queue.pop_front() {
        if depth > 3 {
            continue; // Limit depth to avoid too much output
        }
        
        match volume.list_directory(dir_id) {
            Ok(entries) => {
                for (name, is_file) in entries {
                    let full_path = if path.is_empty() {
                        name.clone()
                    } else {
                        format!("{}:{}", path, name)
                    };
                    
                    let indent = "  ".repeat(depth);
                    let type_str = if is_file { "[FILE]" } else { "[DIR] " };
                    
                    println!("{}{} {}", indent, type_str, full_path);
                    
                    if is_file {
                        total_files += 1;
                        
                        // Check for interesting files
                        let name_lower = name.to_lowercase();
                        if name_lower.contains("system") || 
                           name_lower.contains("boot") ||
                           name_lower.contains("rom") ||
                           name_lower.contains("install") {
                            println!("{}    ^^^ INTERESTING", indent);
                        }
                    } else {
                        total_dirs += 1;
                        // For directories, we'd need to get the dir_id by searching again
                        // For now, just note we can't traverse deeper without that info
                    }
                }
            }
            Err(e) => {
                println!("Error listing directory {}: {}", dir_id, e);
            }
        }
    }
    
    println!("\nTotal: {} directories, {} files", total_dirs, total_files);
}
