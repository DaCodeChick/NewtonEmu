//! Test init-program and go services

use newton_core::{Rom, OpenFirmware};

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("==================================================");
    println!("init-program and go Test");
    println!("==================================================\n");

    // Load ROM
    let rom = Rom::load_from_file("roms/1999-09-17 - Mac OS ROM 2.5.1.rom")
        .expect("Failed to load ROM");

    println!("✓ ROM loaded: {} bytes\n", rom.data().len());

    // Create OpenFirmware instance
    let mut of = OpenFirmware::new();
    println!("✓ OpenFirmware initialized\n");

    // Set up boot environment with constants
    let load_base = 0x00400000u32;
    
    println!("Executing boot sequence simulation...");
    println!("--------------------------------------------------");

    // Simulate the boot script steps
    let forth_code = format!(r#"
        hex
        {:08X} constant load-base
        
        load-base u. cr
        
        init-program
        
        go
    "#, load_base);

    match of.execute_forth(&forth_code) {
        Ok(()) => {
            println!("\n✓ Boot sequence executed successfully!");
            
            // Check if program entry was set
            let forth = of.forth();
            if let Some(entry) = forth.program_entry {
                println!("\n✅ Program entry point set: 0x{:08X}", entry);
                println!("   Ready to transfer control to loaded program!");
            } else {
                println!("\n⚠ No program entry point set");
            }
            
            if let Some(base) = forth.load_base {
                println!("✅ Load base set: 0x{:08X}", base);
            }
        }
        Err(e) => {
            eprintln!("\n✗ Boot sequence failed: {}", e);
        }
    }

    println!("\n==================================================");
    println!("Test Complete");
    println!("==================================================");
    println!("\nSummary:");
    println!("  • init-program: Marks program for initialization");
    println!("  • go: Sets entry point for control transfer");
    println!("  • These services signal to the emulator that it's");
    println!("    time to parse the ELF and jump to its entry point");
}
