// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Inspect ROM script bytes in detail

use newton_core::rom::Rom;

fn main() -> newton_utils::Result<()> {
    let rom = Rom::load_from_file("roms/1998-07-21 - Mac OS ROM 1.1.rom")?;
    let script = rom.get_forth_script().unwrap();
    
    // Find the "find-package" part
    let idx = script.find("find-package").unwrap();
    
    // Show 200 chars around it with escape codes visible
    let start = idx.saturating_sub(50);
    let end = (idx + 200).min(script.len());
    let snippet = &script[start..end];
    
    println!("Around first 'find-package' ({} bytes):", snippet.len());
    println!();
    for (i, byte) in snippet.bytes().enumerate() {
        if byte == b'"' {
            print!("[QUOTE]");
        } else if byte.is_ascii_whitespace() {
            print!("[{}]", match byte {
                b' ' => "SP",
                b'\t' => "TAB",
                b'\n' => "NL",
                b'\r' => "CR",
                _ => "WS",
            });
        } else {
            print!("{}", byte as char);
        }
        if i > 0 && i % 100 == 0 {
            println!();
        }
    }
    println!();
    
    Ok(())
}
