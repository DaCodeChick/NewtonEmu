// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Check if boot script has wrapper around Forth code

use newton_core::rom::Rom;

fn main() -> newton_utils::Result<()> {
    let rom = Rom::load_from_file("roms/1998-07-21 - Mac OS ROM 1.1.rom")?;
    let full_script = rom.get_boot_script().unwrap();
    
    println!("First 500 chars of full CHRP boot data:");
    println!("{}", &full_script[..500.min(full_script.len())]);
    
    println!("\n\n{}\n", "=".repeat(60));
    println!("Last 300 chars:");
    let len = full_script.len();
    println!("{}", &full_script[len.saturating_sub(300)..]);
    
    Ok(())
}
