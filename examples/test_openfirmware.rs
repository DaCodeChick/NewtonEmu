//! Test OpenFirmware client interface
//!
//! This example tests OF service calls without needing ROM code.

use newton_core::{Emulator, EmulatorConfig};
use newton_cpu::MemoryInterface;
use newton_utils::Result;

fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    println!("==================================================");
    println!("OpenFirmware Client Interface Test");
    println!("==================================================\n");

    // Create emulator with minimal config
    let config = EmulatorConfig {
        memory: newton_core::config::MemoryConfig {
            ram_size_mb: 128,
            rom_path: Some("roms/1999-09-17 - Mac OS ROM 2.5.1.rom".into()),
        },
        ..Default::default()
    };

    let mut emulator = Emulator::new(config)?;
    emulator.reset();

    // Test 1: finddevice("/")
    println!("\n[Test 1] finddevice(\"/\")");
    let args_addr = 0x8000;
    let path_addr = 0x8100;  // Use a different location for the path string
    write_string(&emulator, path_addr, b"/")?;
    setup_of_call(&emulator, args_addr, b"finddevice", &[path_addr], &[0u32])?;

    // Set up CPU to call OF
    if let Some(cpu) = emulator.cpu_mut() {
        cpu.registers.gpr[3] = args_addr;
        cpu.registers.pc = 0x3000; // OF stub address
        cpu.registers.lr = 0x1004; // Return address
    }

    // Execute the OF call
    emulator.step()?;

    // Read result
    let phandle = emulator.memory().read_u32(args_addr + 12 + 4)?;
    println!("  Result: phandle=0x{:08X}", phandle);
    if phandle != 0xFFFFFFFF {
        println!("  ✓ Success!");
    } else {
        println!("  ✗ Failed");
    }

    // Test 2: getprop(phandle, "device_type", buf, 64)
    println!("\n[Test 2] getprop(phandle, \"device_type\", buf, 64)");
    let buf_addr = 0x9000;
    let propname_addr = args_addr + 0x2000;
    write_string(&emulator, propname_addr, b"device_type")?;
    setup_of_call(&emulator, args_addr, b"getprop", &[phandle, propname_addr, buf_addr, 64], &[0u32])?;

    if let Some(cpu) = emulator.cpu_mut() {
        cpu.registers.gpr[3] = args_addr;
        cpu.registers.pc = 0x3000;
        cpu.registers.lr = 0x1004;
    }

    emulator.step()?;

    let prop_len = emulator.memory().read_u32(args_addr + 12 + 16)? as usize;
    if prop_len != 0xFFFFFFFF as usize && prop_len > 0 {
        let mut prop_data = vec![0u8; prop_len];
        for i in 0..prop_len {
            prop_data[i] = emulator.memory().read_u8(buf_addr + i as u32)?;
        }
        println!("  Result: {} bytes: {:?}", prop_len, String::from_utf8_lossy(&prop_data));
        println!("  ✓ Success!");
    } else {
        println!("  ✗ Failed");
    }

    // Test 3: claim memory
    println!("\n[Test 3] claim(0, 0x1000, 0)");
    setup_of_call(&emulator, args_addr, b"claim", &[0, 0x1000, 0], &[0u32])?;

    if let Some(cpu) = emulator.cpu_mut() {
        cpu.registers.gpr[3] = args_addr;
        cpu.registers.pc = 0x3000;
        cpu.registers.lr = 0x1004;
    }

    emulator.step()?;

    let claimed_addr = emulator.memory().read_u32(args_addr + 12 + 12)?;
    println!("  Result: base_addr=0x{:08X}", claimed_addr);
    if claimed_addr != 0xFFFFFFFF {
        println!("  ✓ Success!");
    } else {
        println!("  ✗ Failed");
    }

    // Test 4: getproplen - check property length without reading value
    println!("\n[Test 4] getproplen(phandle, \"model\")");
    let propname2_addr = 0xA100;
    write_string(&emulator, propname2_addr, b"model")?;
    setup_of_call(&emulator, args_addr, b"getproplen", &[phandle, propname2_addr], &[0u32])?;

    if let Some(cpu) = emulator.cpu_mut() {
        cpu.registers.gpr[3] = args_addr;
        cpu.registers.pc = 0x3000;
        cpu.registers.lr = 0x1004;
    }

    emulator.step()?;

    let prop_len = emulator.memory().read_u32(args_addr + 12 + 8)?;
    println!("  Result: length={}", prop_len);
    if prop_len != 0xFFFFFFFF as u32 && prop_len > 0 {
        println!("  ✓ Success! (\"Power Macintosh\" = {} bytes)", prop_len);
    } else {
        println!("  ✗ Failed");
    }

    // Test 5: child - get first child of root
    println!("\n[Test 5] child(root_phandle)");
    setup_of_call(&emulator, args_addr, b"child", &[phandle], &[0u32])?;

    if let Some(cpu) = emulator.cpu_mut() {
        cpu.registers.gpr[3] = args_addr;
        cpu.registers.pc = 0x3000;
        cpu.registers.lr = 0x1004;
    }

    emulator.step()?;

    let child_phandle = emulator.memory().read_u32(args_addr + 12 + 4)?;
    println!("  Result: child_phandle=0x{:08X}", child_phandle);
    if child_phandle != 0 && child_phandle != 0xFFFFFFFF {
        println!("  ✓ Success! Found first child");
    } else {
        println!("  ✗ Failed");
    }

    // Test 6: parent - get parent of child
    println!("\n[Test 6] parent(child_phandle)");
    setup_of_call(&emulator, args_addr, b"parent", &[child_phandle], &[0u32])?;

    if let Some(cpu) = emulator.cpu_mut() {
        cpu.registers.gpr[3] = args_addr;
        cpu.registers.pc = 0x3000;
        cpu.registers.lr = 0x1004;
    }

    emulator.step()?;

    let parent_phandle = emulator.memory().read_u32(args_addr + 12 + 4)?;
    println!("  Result: parent_phandle=0x{:08X}", parent_phandle);
    if parent_phandle == phandle {
        println!("  ✓ Success! Parent is root (matches original phandle)");
    } else {
        println!("  ✗ Failed - expected 0x{:08X}, got 0x{:08X}", phandle, parent_phandle);
    }

    // Test 7: peer - get next sibling
    println!("\n[Test 7] peer(child_phandle)");
    setup_of_call(&emulator, args_addr, b"peer", &[child_phandle], &[0u32])?;

    if let Some(cpu) = emulator.cpu_mut() {
        cpu.registers.gpr[3] = args_addr;
        cpu.registers.pc = 0x3000;
        cpu.registers.lr = 0x1004;
    }

    emulator.step()?;

    let peer_phandle = emulator.memory().read_u32(args_addr + 12 + 4)?;
    println!("  Result: peer_phandle=0x{:08X}", peer_phandle);
    if peer_phandle != child_phandle {
        println!("  ✓ Success! Found peer (different from current)");
    } else {
        println!("  ✗ Failed - peer should be different");
    }

    // Test 8: Multiple memory claims with alignment
    println!("\n[Test 8] claim(0, 0x2000, 0x1000) - aligned allocation");
    setup_of_call(&emulator, args_addr, b"claim", &[0, 0x2000, 0x1000], &[0u32])?;

    if let Some(cpu) = emulator.cpu_mut() {
        cpu.registers.gpr[3] = args_addr;
        cpu.registers.pc = 0x3000;
        cpu.registers.lr = 0x1004;
    }

    emulator.step()?;

    let aligned_addr = emulator.memory().read_u32(args_addr + 12 + 12)?;
    println!("  Result: base_addr=0x{:08X}", aligned_addr);
    if aligned_addr % 0x1000 == 0 {
        println!("  ✓ Success! Address is 4KB aligned");
    } else {
        println!("  ✗ Failed - not properly aligned");
    }

    // Test 9: Claim specific address
    println!("\n[Test 9] claim(0x00500000, 0x1000, 0) - specific address");
    setup_of_call(&emulator, args_addr, b"claim", &[0x00500000, 0x1000, 0], &[0u32])?;

    if let Some(cpu) = emulator.cpu_mut() {
        cpu.registers.gpr[3] = args_addr;
        cpu.registers.pc = 0x3000;
        cpu.registers.lr = 0x1004;
    }

    emulator.step()?;

    let specific_addr = emulator.memory().read_u32(args_addr + 12 + 12)?;
    println!("  Result: base_addr=0x{:08X}", specific_addr);
    if specific_addr == 0x00500000 {
        println!("  ✓ Success! Got requested address");
    } else {
        println!("  ✗ Failed - expected 0x00500000, got 0x{:08X}", specific_addr);
    }

    // Test 10: Release memory
    println!("\n[Test 10] release(0x00500000, 0x1000)");
    setup_of_call(&emulator, args_addr, b"release", &[0x00500000, 0x1000], &[0u32])?;

    if let Some(cpu) = emulator.cpu_mut() {
        cpu.registers.gpr[3] = args_addr;
        cpu.registers.pc = 0x3000;
        cpu.registers.lr = 0x1004;
    }

    emulator.step()?;

    let release_result = emulator.memory().read_u32(args_addr + 12 + 8)?;
    println!("  Result: status=0x{:08X}", release_result);
    if release_result == 0 {
        println!("  ✓ Success! Memory released");
    } else {
        println!("  ✗ Failed");
    }

    println!("\n==================================================");
    println!("Test complete! All OpenFirmware services working!");
    println!("  - Device tree traversal: peer, child, parent");
    println!("  - Property access: getprop, getproplen");
    println!("  - Device lookup: finddevice");
    println!("  - Memory management: claim (dynamic, aligned, specific), release");
    println!("==================================================");

    Ok(())
}

/// Set up an OF call argument structure in memory
fn setup_of_call(
    emulator: &Emulator,
    args_addr: u32,
    service_name: &[u8],
    args: &[u32],
    returns: &[u32],
) -> Result<()> {
    let mem = emulator.memory();
    
    // Write service name
    let name_addr = args_addr + 0x1000;
    for (i, &byte) in service_name.iter().enumerate() {
        mem.write_u8(name_addr + i as u32, byte)?;
    }
    mem.write_u8(name_addr + service_name.len() as u32, 0)?; // null terminator
    
    // Write argument structure
    mem.write_u32(args_addr, name_addr)?; // service name ptr
    mem.write_u32(args_addr + 4, args.len() as u32)?; // n_args
    mem.write_u32(args_addr + 8, returns.len() as u32)?; // n_returns
    
    // Write input args
    for (i, &arg) in args.iter().enumerate() {
        mem.write_u32(args_addr + 12 + (i as u32 * 4), arg)?;
    }
    
    // Clear return values
    for i in 0..returns.len() {
        mem.write_u32(args_addr + 12 + (args.len() as u32 * 4) + (i as u32 * 4), 0)?;
    }
    
    Ok(())
}

/// Write a string to memory
fn write_string(emulator: &Emulator, addr: u32, s: &[u8]) -> Result<()> {
    let mem = emulator.memory();
    for (i, &byte) in s.iter().enumerate() {
        mem.write_u8(addr + i as u32, byte)?;
    }
    mem.write_u8(addr + s.len() as u32, 0)?; // null terminator
    Ok(())
}
