// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Test execution of ROM boot scripts

use newton_core::rom::Rom;
use newton_core::openfirmware::forth::ForthInterpreter;
use newton_core::openfirmware::forth_of_words::OpenFirmwareForthExt;

fn main() -> newton_utils::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // Load the earliest/simplest ROM
    let rom_path = "roms/1998-07-21 - Mac OS ROM 1.1.rom";
    
    println!("Loading ROM: {}", rom_path);
    let rom = Rom::load_from_file(rom_path)?;
    
    println!("ROM type: {:?}", rom.rom_type());
    println!("ROM size: {} bytes", rom.size());
    
    // Get the Forth script (not the full CHRP boot data with XML)
    let script = rom.get_forth_script()
        .ok_or_else(|| newton_utils::Error::Other("No Forth script found".to_string()))?;
    
    println!("\nForth script size: {} bytes", script.len());
    println!("\nFirst 500 characters:");
    println!("{}", &script[..500.min(script.len())]);
    println!("\n...\n");
    
    // Create Forth interpreter
    let mut forth = ForthInterpreter::new();
    forth.register_of_words();
    
    // Set up some stub values that boot scripts expect
    println!("Setting up environment...");
    
    // Simulate load-base (where ROM data is loaded)
    forth.eval("hex 600000 constant load-base")?;
    
    // Simulate load-size (size of loaded data)  
    forth.eval("100000 constant load-size")?;
    
    // Add some common OpenFirmware words as stubs
    // abort - print message and stop
    forth.eval(": abort cr type cr quit ;")?;
    
    // cr - carriage return (newline)
    forth.eval(": cr ;")?;  // Stub: do nothing
    
    // type - print string
    forth.eval(": type 2drop ;")?;  // Stub: just drop addr/len
    
    // . - print number
    forth.eval(": . drop ;")?;  // Stub: just drop number
    
    // .\" - print quoted string
    forth.eval(": .\" ;")? ;  // Stub: do nothing
    
    // quit - exit
    forth.eval(": quit ;")?;  // Stub: do nothing
    
    // u. - print unsigned number
    forth.eval(": u. drop ;")?;
    
    // allot - allocate space
    forth.eval(": allot drop ;")?;
    
    // c@ - fetch character (already have this but let's make sure)
    // @ - fetch word (already have this)
    
    // bounds - convert addr/len to end/start for do loops
    // already implemented in OF words
    
    println!("Environment ready.\n");
    
    // Try to execute the boot script line by line to see where it fails
    println!("\nExecuting boot script (line by line)...");
    println!("{}", "=".repeat(60));
    
    let mut line_num = 0;
    let mut failed_at_line = None;
    
    for line in script.lines() {
        line_num += 1;
        let trimmed = line.trim();
        
        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('\\') {
            continue;
        }
        
        // Try to execute this line
        match forth.eval(trimmed) {
            Ok(()) => {
                // Success - continue
                if line_num <= 10 {
                    println!("✓ Line {}: {}", line_num, &trimmed[..trimmed.len().min(60)]);
                }
            }
            Err(e) => {
                println!("\n✗ Failed at line {}: {}", line_num, e);
                println!("   Line content: {}", trimmed);
                failed_at_line = Some(line_num);
                break;
            }
        }
    }
    
    println!("\n{}", "=".repeat(60));
    
    if let Some(line) = failed_at_line {
        println!("Execution stopped at line {}/{}", line, line_num);
        println!("This shows which Forth words need to be implemented.");
    } else {
        println!("✓ All {} lines executed successfully!", line_num);
        println!("\nFinal stack depth: {}", forth.depth());
    }
    
    Ok(())
}
