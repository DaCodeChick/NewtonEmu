// Dump decoded ROM to inspect its contents

use newton_core::rom::Rom;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rom_path = "roms/1999-09-17 - Mac OS ROM 2.5.1.rom";
    
    println!("Loading ROM: {}", rom_path);
    let rom = Rom::load_from_file(rom_path)?;
    
    println!("ROM size: {} bytes", rom.size());
    println!("ROM base: 0x{:08X}", rom.base_address());
    
    // Dump first 256 bytes
    println!("\nFirst 256 bytes of decoded ROM:");
    for (i, chunk) in rom.data().chunks(16).take(16).enumerate() {
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
    
    // Check for PowerPC code patterns
    println!("\nSearching for PowerPC code patterns...");
    let data = rom.data();
    for offset in (0..4096).step_by(4) {
        if offset + 4 <= data.len() {
            let word = u32::from_be_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            
            // Look for common PowerPC instructions
            if (word & 0xFC000000) == 0x7C000000 || // mfspr, mtspr, etc.
               (word & 0xFC000000) == 0x48000000 || // b, bl
               (word & 0xFC000000) == 0x94000000 || // stwu
               (word & 0xF0000000) == 0x30000000 {  // lis, li
                println!("  Possible PowerPC code at 0x{:08X}: 0x{:08X}", offset, word);
                if offset > 0 {
                    break;
                }
            }
        }
    }
    
    // Save decoded ROM to file for inspection
    let output = "/tmp/decoded_rom.bin";
    let mut file = std::fs::File::create(output)?;
    file.write_all(rom.data())?;
    println!("\nDecoded ROM saved to: {}", output);
    
    Ok(())
}
