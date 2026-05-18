//! Test ELF loader with Mac ROM embedded ELF

use newton_core::{Rom, ElfFile};
use std::fs;

fn main() {
    println!("==================================================");
    println!("ELF Loader Test - Mac ROM Embedded ELF");
    println!("==================================================\n");

    // Load ROM
    let rom = Rom::load_from_file("roms/1999-09-17 - Mac OS ROM 2.5.1.rom")
        .expect("Failed to load ROM");

    println!("✓ ROM loaded: {} bytes\n", rom.data().len());

    // Extract ELF from ROM
    // Boot script constants:
    // h# 004000 constant elf-offset
    // h# 011690 constant elf-size
    let elf_offset = 0x4000;
    let elf_size = 0x11690;

    println!("Extracting ELF from ROM:");
    println!("  Offset: 0x{:X}", elf_offset);
    println!("  Size:   0x{:X} ({} bytes)", elf_size, elf_size);

    let elf_data = rom.data()[elf_offset..elf_offset + elf_size].to_vec();

    println!("  ✓ Extracted {} bytes\n", elf_data.len());

    // Parse ELF
    println!("Parsing ELF...");
    let elf = ElfFile::parse(elf_data).expect("Failed to parse ELF");

    println!("  ✓ Valid ELF file\n");

    println!("ELF Header:");
    println!("  Class:        {:?}", elf.header.class);
    println!("  Data:         {:?}", elf.header.data);
    println!("  Type:         {:?}", elf.header.file_type);
    println!("  Machine:      {:?}", elf.header.machine);
    println!("  Entry point:  0x{:08X}", elf.header.entry);
    println!("  Flags:        0x{:08X}", elf.header.flags);
    println!();

    println!("Program Headers: {} entries", elf.program_headers.len());
    for (i, ph) in elf.program_headers.iter().enumerate() {
        let type_name = match ph.p_type {
            0 => "NULL",
            1 => "LOAD",
            2 => "DYNAMIC",
            3 => "INTERP",
            4 => "NOTE",
            5 => "SHLIB",
            6 => "PHDR",
            _ => "UNKNOWN",
        };
        
        println!("  [{}] {} (type={})", i, type_name, ph.p_type);
        println!("      vaddr:  0x{:08X}", ph.vaddr);
        println!("      paddr:  0x{:08X}", ph.paddr);
        println!("      filesz: 0x{:X} ({} bytes)", ph.filesz, ph.filesz);
        println!("      memsz:  0x{:X} ({} bytes)", ph.memsz, ph.memsz);
        println!("      flags:  0x{:X}", ph.flags);
        println!("      align:  0x{:X}", ph.align);
    }
    println!();

    // Load ELF to memory
    println!("Loading ELF to memory...");
    let (memory, load_addr, entry) = elf.load_to_memory().expect("Failed to load ELF");
    
    println!("  ✓ Loaded {} bytes", memory.len());
    println!("  Load address: 0x{:08X}", load_addr);
    println!("  Entry point:  0x{:08X}", entry);
    println!();

    // Verify entry point code
    if entry >= load_addr && (entry - load_addr) < memory.len() as u32 {
        let entry_offset = (entry - load_addr) as usize;
        if entry_offset + 16 <= memory.len() {
            println!("Code at entry point:");
            print!("  ");
            for i in 0..16 {
                print!("{:02X} ", memory[entry_offset + i]);
                if i % 4 == 3 {
                    print!(" ");
                }
            }
            println!();
            
            // Decode first instruction
            let first_instr = u32::from_be_bytes([
                memory[entry_offset],
                memory[entry_offset + 1],
                memory[entry_offset + 2],
                memory[entry_offset + 3],
            ]);
            println!("  First instruction: 0x{:08X}", first_instr);
        }
    }

    println!("\n==================================================");
    println!("ELF Loader Test Complete!");
    println!("==================================================");
    println!("\nSummary:");
    println!("  • Successfully extracted ELF from Mac ROM");
    println!("  • Parsed valid PowerPC ELF executable");
    println!("  • Entry point: 0x{:08X}", entry);
    println!("  • {} loadable segments", 
             elf.program_headers.iter().filter(|ph| ph.p_type == 1).count());
    println!("  • Total memory footprint: {} KB", memory.len() / 1024);
}
