//! Test CHRP boot script parsing

use newton_core::{Rom, RomType};
use newton_core::openfirmware::BootScriptParser;
use newton_utils::Result;

fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("==================================================");
    println!("CHRP Boot Script Parser Test");
    println!("==================================================\n");

    // Load the ROM
    let rom = Rom::load_from_file("roms/1999-09-17 - Mac OS ROM 2.5.1.rom")?;

    if rom.rom_type() == RomType::NewWorld {
        println!("✓ NewWorld ROM detected\n");
        
        if let Some(script) = rom.get_boot_script() {
            println!("Boot script extracted ({} bytes):", script.len());
            println!("--------------------------------------------------");
            
            // Show first few lines
            for (i, line) in script.lines().take(20).enumerate() {
                println!("{:3}: {}", i + 1, line);
            }
            
            if script.lines().count() > 20 {
                println!("... ({} more lines)", script.lines().count() - 20);
            }
            
            println!("--------------------------------------------------\n");
            
            // Parse the boot script
            let parser = BootScriptParser::new(script);
            let boot_info = parser.parse()?;
            
            println!("\nParsed boot information:");
            println!("--------------------------------------------------");
            
            if let Some(ref device) = boot_info.boot_device {
                println!("Boot device:  {}", device);
            } else {
                println!("Boot device:  <not specified>");
            }
            
            if let Some(ref file) = boot_info.boot_file {
                println!("Boot file:    {}", file);
            } else {
                println!("Boot file:    <not specified>");
            }
            
            if let Some(base) = boot_info.load_base {
                println!("Load base:    0x{:08X}", base);
            } else {
                println!("Load base:    <not specified>");
            }
            
            if boot_info.is_bootable() {
                println!("\n✓ Boot script contains bootable configuration");
            } else {
                println!("\n⚠ Boot script lacks boot configuration");
            }
            
        } else {
            println!("✗ Failed to extract boot script from ROM");
        }
    } else {
        println!("✗ Not a NewWorld ROM - no CHRP boot script");
    }

    println!("\n==================================================");
    println!("Test complete!");
    println!("==================================================");

    Ok(())
}
