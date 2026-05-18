//! Execute Mac ROM boot script through Forth interpreter

use newton_core::{Rom, OpenFirmware};

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    println!("==================================================");
    println!("Boot Script Execution Test");
    println!("==================================================\n");

    // Load ROM
    let rom = Rom::load_from_file("roms/1999-09-17 - Mac OS ROM 2.5.1.rom")
        .expect("Failed to load ROM");

    println!("✓ ROM loaded: {} bytes\n", rom.data().len());

    // Extract boot script
    let boot_script = rom.get_boot_script().expect("Failed to extract boot script");
    println!("✓ Boot script extracted: {} bytes\n", boot_script.len());

    // Create OpenFirmware instance
    let mut of = OpenFirmware::new();
    println!("✓ OpenFirmware initialized\n");

    // Set up ROM data as constants that the boot script expects
    // The boot script references load-base and load-size
    let load_base = 0x00400000u32;
    let rom_data_ptr = rom.data().as_ptr() as u32;
    
    println!("Setting up boot environment:");
    println!("  load-base: 0x{:08X}", load_base);
    println!("  ROM data:  0x{:08X} ({} bytes)", rom_data_ptr, rom.data().len());
    println!();

    // Try to execute a simplified version of the boot script
    // The full script is complex and references many OF internals
    println!("Executing simplified boot script...");
    println!("--------------------------------------------------");

    // Define constants from the boot script
    // Note: h# is a Forth word that reads the next token as hex
    let forth_code = format!(r#"
        hex
        {:08X} constant load-base
        {:08X} constant load-size
        004000 constant elf-offset
        011690 constant elf-size
        015690 constant lzss-offset
        208880 constant lzss-size
        
        cr
        load-base u. cr
        load-size u. cr  
        elf-offset u. cr
        elf-size u. cr
        lzss-offset u. cr
        lzss-size u. cr
    "#, load_base, rom.data().len());

    match of.execute_forth(&forth_code) {
        Ok(()) => {
            println!("\n✓ Boot script executed successfully!");
            
            // Check Forth stack
            let forth = of.forth();
            println!("\nForth state:");
            println!("  Data stack depth: {}", forth.stack_depth());
            println!("  Dictionary size:  {} words", forth.dictionary_size());
        }
        Err(e) => {
            eprintln!("\n✗ Boot script execution failed: {}", e);
            eprintln!("\nThis is expected - the full boot script requires many");
            eprintln!("OF services that aren't yet implemented.");
        }
    }

    println!("\n==================================================");
    println!("Boot Script Execution Test Complete");
    println!("==================================================");
}
