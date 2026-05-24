// Analyze the raw NewWorld ROM structure before decompression

use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rom_path = "roms/1999-09-17 - Mac OS ROM 2.5.1.rom";
    
    println!("Loading RAW ROM (before decompression): {}", rom_path);
    let raw_data = fs::read(rom_path)?;
    
    println!("Raw ROM size: {} bytes", raw_data.len());
    println!();
    
    // Check for CHRP header
    if &raw_data[0..11] == b"<CHRP-BOOT>" {
        println!("✓ Found CHRP-BOOT header");
    }
    
    // Look for constants in boot script
    let script_str = String::from_utf8_lossy(&raw_data[0..65536.min(raw_data.len())]);
    
    println!("\n=== Boot Script Constants ===");
    for line in script_str.lines() {
        if line.contains("constant") && (line.contains("offset") || line.contains("size") || line.contains("entry")) {
            println!("  {}", line.trim());
        }
    }
    
    // Look for ELF header
    println!("\n=== Searching for ELF Header ===");
    for offset in (0..65536).step_by(4) {
        if offset + 4 <= raw_data.len() {
            if &raw_data[offset..offset+4] == b"\x7FELF" {
                println!("✓ Found ELF header at offset 0x{:X}", offset);
                
                // Parse ELF header basics
                if offset + 24 <= raw_data.len() {
                    let entry_offset = u32::from_be_bytes([
                        raw_data[offset + 24],
                        raw_data[offset + 25],
                        raw_data[offset + 26],
                        raw_data[offset + 27],
                    ]);
                    println!("  ELF entry point: 0x{:08X}", entry_offset);
                }
                break;
            }
        }
    }
    
    // Look for PowerPC code patterns in the region around 0x4000-0x5000
    println!("\n=== Scanning 0x4000-0x5000 for PowerPC Code ===");
    let search_start = 0x4000;
    let search_end = 0x5000.min(raw_data.len());
    
    for offset in (search_start..search_end).step_by(4) {
        if offset + 4 <= raw_data.len() {
            let word = u32::from_be_bytes([
                raw_data[offset],
                raw_data[offset + 1],
                raw_data[offset + 2],
                raw_data[offset + 3],
            ]);
            
            // Look for common PowerPC instruction patterns at start of code
            let looks_like_ppc = 
                (word & 0xFC000000) == 0x7C000000 || // mfspr, mtspr, mfmsr, mtmsr
                (word & 0xFC000000) == 0x48000000 || // b, bl
                (word & 0xFC000000) == 0x94000000 || // stwu
                (word & 0xFC000000) == 0x90000000 || // stw
                (word & 0xF0000000) == 0x30000000 || // lis, li
                word == 0x4E800020; // blr
                
            if looks_like_ppc {
                println!("  0x{:04X}: 0x{:08X}", offset, word);
                
                // Check next few instructions
                let mut all_valid = true;
                for i in 1..5 {
                    if offset + i * 4 + 4 <= raw_data.len() {
                        let next_word = u32::from_be_bytes([
                            raw_data[offset + i * 4],
                            raw_data[offset + i * 4 + 1],
                            raw_data[offset + i * 4 + 2],
                            raw_data[offset + i * 4 + 3],
                        ]);
                        
                        // Check if it's a reasonable PowerPC instruction
                        if next_word == 0 || next_word == 0xFFFFFFFF {
                            all_valid = false;
                            break;
                        }
                        
                        println!("         +{:02}: 0x{:08X}", i*4, next_word);
                    }
                }
                
                if all_valid {
                    println!("  ^^^ Looks like valid PowerPC code!");
                }
                
                // Only show first match
                break;
            }
        }
    }
    
    // Dump first 512 bytes of raw ROM
    println!("\n=== First 512 Bytes of Raw ROM ===");
    for (i, chunk) in raw_data.chunks(16).take(32).enumerate() {
        print!("{:08X}: ", i * 16);
        for byte in chunk {
            print!("{:02X} ", byte);
        }
        print!(" | ");
        for byte in chunk {
            let c = if byte.is_ascii_graphic() || *byte == b' ' {
                *byte as char
            } else {
                '.'
            };
            print!("{}", c);
        }
        println!();
    }
    
    Ok(())
}
