// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Dump raw bytes around the problematic constant definition to check for missing dup

use newton_core::rom::Rom;

fn main() -> newton_utils::Result<()> {
    let rom = Rom::load_from_file("roms/1998-07-21 - Mac OS ROM 1.1.rom")?;
    let script = rom.get_forth_script().unwrap();
    
    // Find the problematic section: "find-package 0= abort" ... "constant /chosen"
    let idx = script.find("constant /chosen").unwrap();
    
    // Show 100 chars before "constant /chosen"
    let start = idx.saturating_sub(100);
    let end = idx + 30;
    let snippet = &script[start..end];
    
    println!("Raw bytes around 'constant /chosen':");
    println!("Position {}-{} in script", start, idx + 30);
    println!();
    
    // Show as both hex and text
    for (i, byte) in snippet.bytes().enumerate() {
        let pos = start + i;
        if pos == idx {
            println!("\n>>> HERE: constant /chosen starts <<<");
        }
        
        let display_char = if byte.is_ascii_graphic() || byte == b' ' {
            format!("'{}'", byte as char)
        } else if byte == b'\r' {
            "'\\r'".to_string()
        } else if byte == b'\n' {
            "'\\n'".to_string()
        } else if byte == b'\t' {
            "'\\t'".to_string()
        } else {
            format!("0x{:02X}", byte)
        };
        
        print!("{:5}: {:02X} {:>5}  ", pos, byte, display_char);
        
        if (i + 1) % 3 == 0 {
            println!();
        }
    }
    println!();
    
    // Now specifically look for "dup" between "find-package" and "0="
    println!("\n\nSearching for 'dup' after 'find-package'...");
    if let Some(fp_idx) = script.find("find-package") {
        let search_area = &script[fp_idx..fp_idx + 100];
        println!("Text after 'find-package': {:?}", search_area);
        
        if search_area.contains("dup") {
            println!("✓ Found 'dup' in this section!");
        } else {
            println!("✗ No 'dup' found in this section");
        }
    }
    
    // Check the second instance (after /chosen is defined)
    println!("\n\nSearching for 'dup' in memory property lookup...");
    if let Some(mem_idx) = script.find("\" memory\"") {
        let search_area = &script[mem_idx..mem_idx.saturating_add(80).min(script.len())];
        println!("Text: {:?}", search_area);
        
        if search_area.contains("dup") {
            println!("✓ Found 'dup' before abort");
        } else {
            println!("✗ No 'dup' found - this may be the issue!");
        }
    }
    
    Ok(())
}
