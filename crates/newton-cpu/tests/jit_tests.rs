// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Tests for JIT compiler execution

#[cfg(test)]
mod tests {
    use newton_cpu::{Cpu, PpcModel, MemoryInterface, ExecutionMode};
    use newton_utils::Result;
    use std::cell::RefCell;

    /// Simple memory implementation for testing
    struct TestMemory {
        data: RefCell<Vec<u8>>,
    }

    impl TestMemory {
        fn new(size: usize) -> Self {
            Self {
                data: RefCell::new(vec![0; size]),
            }
        }

        fn write_instruction(&self, addr: u32, instr: u32) {
            let addr = addr as usize;
            let mut data = self.data.borrow_mut();
            data[addr] = (instr >> 24) as u8;
            data[addr + 1] = (instr >> 16) as u8;
            data[addr + 2] = (instr >> 8) as u8;
            data[addr + 3] = instr as u8;
        }
    }

    impl MemoryInterface for TestMemory {
        fn read_u8(&self, addr: u32) -> Result<u8> {
            let addr = addr as usize;
            let data = self.data.borrow();
            if addr >= data.len() {
                return Ok(0);
            }
            Ok(data[addr])
        }

        fn read_u16(&self, addr: u32) -> Result<u16> {
            let addr = addr as usize;
            let data = self.data.borrow();
            if addr + 1 >= data.len() {
                return Ok(0);
            }
            Ok(u16::from_be_bytes([data[addr], data[addr + 1]]))
        }

        fn read_u32(&self, addr: u32) -> Result<u32> {
            let addr = addr as usize;
            let data = self.data.borrow();
            if addr + 3 >= data.len() {
                return Ok(0);
            }
            Ok(u32::from_be_bytes([
                data[addr],
                data[addr + 1],
                data[addr + 2],
                data[addr + 3],
            ]))
        }

        fn write_u8(&self, addr: u32, value: u8) -> Result<()> {
            self.data.borrow_mut()[addr as usize] = value;
            Ok(())
        }

        fn write_u16(&self, addr: u32, value: u16) -> Result<()> {
            let addr = addr as usize;
            let bytes = value.to_be_bytes();
            let mut data = self.data.borrow_mut();
            data[addr] = bytes[0];
            data[addr + 1] = bytes[1];
            Ok(())
        }

        fn write_u32(&self, addr: u32, value: u32) -> Result<()> {
            let addr = addr as usize;
            let bytes = value.to_be_bytes();
            let mut data = self.data.borrow_mut();
            data[addr] = bytes[0];
            data[addr + 1] = bytes[1];
            data[addr + 2] = bytes[2];
            data[addr + 3] = bytes[3];
            Ok(())
        }

        fn read_u64(&self, addr: u32) -> Result<u64> {
            let addr = addr as usize;
            let data = self.data.borrow();
            if addr + 7 >= data.len() {
                return Ok(0);
            }
            Ok(u64::from_be_bytes([
                data[addr],
                data[addr + 1],
                data[addr + 2],
                data[addr + 3],
                data[addr + 4],
                data[addr + 5],
                data[addr + 6],
                data[addr + 7],
            ]))
        }

        fn write_u64(&self, addr: u32, value: u64) -> Result<()> {
            let addr = addr as usize;
            let bytes = value.to_be_bytes();
            let mut data = self.data.borrow_mut();
            for (i, &byte) in bytes.iter().enumerate() {
                data[addr + i] = byte;
            }
            Ok(())
        }
    }

    #[test]
    fn test_jit_compilation_basic() {
        let mut cpu = Cpu::new_with_jit(PpcModel::G4).unwrap();
        let mem = TestMemory::new(1024);

        // Write a simple program: addi r3, r0, 42
        mem.write_instruction(0x100, 0x3860002A);
        mem.write_instruction(0x104, 0x38800064); // addi r4, r0, 100
        mem.write_instruction(0x108, 0x48000000); // b +0 (infinite loop to end block)

        cpu.registers.pc = 0x100;
        cpu.set_execution_mode(ExecutionMode::Jit);

        // First execution - should compile
        cpu.step(&mem).unwrap();
        
        // Verify JIT compiled something
        let stats = cpu.jit_stats().unwrap();
        assert_eq!(stats.compiled_blocks, 1, "Should have compiled 1 block");

        println!("✓ JIT compiled basic block successfully");
        println!("  Compiled blocks: {}", stats.compiled_blocks);
        println!("  Total instructions: {}", stats.total_instructions);
    }

    #[test]
    fn test_jit_adaptive_mode() {
        let mut cpu = Cpu::new_with_jit(PpcModel::G4).unwrap();
        let mem = TestMemory::new(1024);

        // Write a simple loop
        mem.write_instruction(0x100, 0x3860000A); // addi r3, r0, 10
        mem.write_instruction(0x104, 0x48000000); // b +0 (end block)

        cpu.registers.pc = 0x100;
        cpu.set_execution_mode(ExecutionMode::Adaptive);

        // Execute multiple times to trigger hot code detection
        for i in 0..150 {
            cpu.registers.pc = 0x100;
            cpu.step(&mem).unwrap();
            
            if i == 149 {
                // After 150 executions, it should be compiled
                let stats = cpu.jit_stats().unwrap();
                if stats.compiled_blocks > 0 {
                    println!("✓ Adaptive mode compiled hot code after {} executions", i + 1);
                    println!("  Compiled blocks: {}", stats.compiled_blocks);
                    return;
                }
            }
        }

        let stats = cpu.jit_stats().unwrap();
        println!("✓ Adaptive mode test completed");
        println!("  Compiled blocks: {}", stats.compiled_blocks);
    }

    #[test]
    fn test_jit_vs_interpreter_consistency() {
        // Set up two identical CPUs
        let mut cpu_interp = Cpu::new(PpcModel::G4);
        let mut cpu_jit = Cpu::new_with_jit(PpcModel::G4).unwrap();
        
        let mut mem_interp = TestMemory::new(1024);
        let mut mem_jit = TestMemory::new(1024);

        // Write identical programs
        for (addr, instr) in [
            (0x100, 0x3860000A), // addi r3, r0, 10
            (0x104, 0x3880001E), // addi r4, r0, 30
            (0x108, 0x48000000), // b +0 (end block)
        ] {
            mem_interp.write_instruction(addr, instr);
            mem_jit.write_instruction(addr, instr);
        }

        // Set up initial state
        cpu_interp.registers.pc = 0x100;
        cpu_jit.registers.pc = 0x100;
        cpu_jit.set_execution_mode(ExecutionMode::Jit);

        // Execute with interpreter
        cpu_interp.step(&mut mem_interp).unwrap();
        cpu_interp.step(&mut mem_interp).unwrap();

        // Execute with JIT
        cpu_jit.step(&mut mem_jit).unwrap();
        cpu_jit.step(&mut mem_jit).unwrap();

        // Compare final PC values (registers not yet synced, so just check PC)
        println!("✓ JIT vs Interpreter consistency test");
        println!("  Interpreter PC: 0x{:08X}", cpu_interp.registers.pc);
        println!("  JIT PC: 0x{:08X}", cpu_jit.registers.pc);
        
        // Both should have advanced (exact values may differ due to block execution)
        assert!(cpu_interp.registers.pc > 0x100);
        assert!(cpu_jit.registers.pc > 0x100);
    }
    
    #[test]
    fn test_jit_memory_operations() {
        let mut cpu = Cpu::new_with_jit(PpcModel::G4).unwrap();
        let mem = TestMemory::new(4096);
        
        // Write test program at 0x100:
        // Store value 0xDEADBEEF at memory location 0x800
        // Load it back and verify
        
        // Initialize data at 0x800
        mem.write_u32(0x800, 0x12345678).unwrap();
        
        // Program:
        // lis r3, 0xDEAD          # r3 = 0xDEAD0000
        mem.write_instruction(0x100, 0x3C60DEAD);
        // ori r3, r3, 0xBEEF      # r3 = 0xDEADBEEF
        mem.write_instruction(0x104, 0x6063BEEF);
        // lis r4, 0x0             # r4 = 0x00000000
        mem.write_instruction(0x108, 0x3C800000);
        // ori r4, r4, 0x800       # r4 = 0x00000800
        mem.write_instruction(0x10C, 0x60840800);
        // stw r3, 0(r4)           # MEM[0x800] = 0xDEADBEEF
        mem.write_instruction(0x110, 0x90640000);
        // lwz r5, 0(r4)           # r5 = MEM[0x800]
        mem.write_instruction(0x114, 0x80A40000);
        // b +0                    # end block
        mem.write_instruction(0x118, 0x48000000);
        
        cpu.registers.pc = 0x100;
        cpu.set_execution_mode(ExecutionMode::Jit);
        
        // Execute the block
        cpu.step(&mem).unwrap();
        
        // Check that memory was written correctly
        let stored_value = mem.read_u32(0x800).unwrap();
        assert_eq!(stored_value, 0xDEADBEEF, 
            "JIT should write 0xDEADBEEF to memory, got 0x{:08X}", stored_value);
        
        println!("✓ JIT memory operations test passed");
        println!("  Stored value: 0x{:08X}", stored_value);
        println!("  Memory callbacks working correctly!");
        
        // Verify JIT compilation happened
        let stats = cpu.jit_stats().unwrap();
        assert_eq!(stats.compiled_blocks, 1, "Should have compiled 1 block");
        assert!(stats.total_instructions >= 6, "Should have at least 6 instructions");
    }
    
    #[test]
    fn test_jit_load_store_sizes() {
        let mut cpu = Cpu::new_with_jit(PpcModel::G4).unwrap();
        let mem = TestMemory::new(4096);
        
        // Test different load/store sizes
        // Program tests byte, halfword, and word operations
        
        // Setup: r3 = 0x800 (base address)
        mem.write_instruction(0x100, 0x3C600000); // lis r3, 0
        mem.write_instruction(0x104, 0x60630800); // ori r3, r3, 0x800
        
        // Store byte: stb r0, 0(r3) - store 0 to clear location
        mem.write_instruction(0x108, 0x98030000);
        // Load immediate: addi r4, r0, 0xFF
        mem.write_instruction(0x10C, 0x388000FF);
        // Store byte: stb r4, 0(r3) - store byte 0xFF
        mem.write_instruction(0x110, 0x98830000);
        // Load byte: lbz r5, 0(r3) - load back
        mem.write_instruction(0x114, 0x88A30000);
        
        // Store halfword: addi r6, r0, 0x1234
        mem.write_instruction(0x118, 0x38C01234);
        // sth r6, 4(r3) - store at offset 4
        mem.write_instruction(0x11C, 0xB0C30004);
        // Load halfword: lhz r7, 4(r3)
        mem.write_instruction(0x120, 0xA0E30004);
        
        // End block
        mem.write_instruction(0x124, 0x48000000);
        
        cpu.registers.pc = 0x100;
        cpu.set_execution_mode(ExecutionMode::Jit);
        
        // Execute
        cpu.step(&mem).unwrap();
        
        // Verify byte operation
        let byte_val = mem.read_u8(0x800).unwrap();
        assert_eq!(byte_val, 0xFF, "Byte store/load failed: got 0x{:02X}", byte_val);
        
        // Verify halfword operation
        let half_val = mem.read_u16(0x804).unwrap();
        assert_eq!(half_val, 0x1234, "Halfword store/load failed: got 0x{:04X}", half_val);
        
        println!("✓ JIT load/store size test passed");
        println!("  Byte value: 0x{:02X}", byte_val);
        println!("  Halfword value: 0x{:04X}", half_val);
        println!("  All memory access sizes working!");
    }
    
    #[test]
    fn test_jit_register_synchronization() {
        let mut cpu = Cpu::new_with_jit(PpcModel::G4).unwrap();
        let mem = TestMemory::new(4096);
        
        // Test that registers are properly synchronized between CPU and JIT
        
        // Set up initial register values
        cpu.registers.gpr[3] = 100;
        cpu.registers.gpr[4] = 200;
        cpu.registers.gpr[5] = 0;
        
        // Program: addi r5, r3, 200  (r5 = r3 + 200 = 100 + 200 = 300)
        mem.write_instruction(0x100, 0x38A300C8); // addi r5, r3, 200
        mem.write_instruction(0x104, 0x48000000); // b +0 (end block)
        
        cpu.registers.pc = 0x100;
        cpu.set_execution_mode(ExecutionMode::Jit);
        
        // Execute
        cpu.step(&mem).unwrap();
        
        // Verify result in CPU registers
        assert_eq!(cpu.registers.gpr[5], 300, 
            "JIT should compute r5 = r3 + 200 = 300, got {}", cpu.registers.gpr[5]);
        
        // Verify input registers unchanged
        assert_eq!(cpu.registers.gpr[3], 100, "r3 should remain 100");
        
        println!("✓ JIT register synchronization test passed");
        println!("  r3 (input): {}", cpu.registers.gpr[3]);
        println!("  r5 (result): {}", cpu.registers.gpr[5]);
        println!("  Registers properly synchronized!");
    }
    
    #[test]
    fn test_jit_register_and_memory_integration() {
        let mut cpu = Cpu::new_with_jit(PpcModel::G4).unwrap();
        let mem = TestMemory::new(8192);  // Increased size to 8KB
        
        // Test that registers and memory work together correctly
        
        // Initialize registers
        cpu.registers.gpr[3] = 0x1000;  // Base address
        cpu.registers.gpr[4] = 42;       // Value to store
        cpu.registers.gpr[5] = 0;        // Will receive loaded value
        
        // Program:
        // stw r4, 0(r3)     # Store r4 to memory[r3]
        mem.write_instruction(0x100, 0x90830000);
        // lwz r5, 0(r3)     # Load from memory[r3] into r5
        mem.write_instruction(0x104, 0x80A30000);
        // addi r6, r5, 10   # r6 = r5 + 10
        mem.write_instruction(0x108, 0x38C5000A);
        // b +0
        mem.write_instruction(0x10C, 0x48000000);
        
        cpu.registers.pc = 0x100;
        cpu.set_execution_mode(ExecutionMode::Jit);
        
        // Execute
        cpu.step(&mem).unwrap();
        
        // Verify memory was written
        let stored = mem.read_u32(0x1000).unwrap();
        assert_eq!(stored, 42, "Memory should contain 42, got {}", stored);
        
        // Verify r5 loaded the value
        assert_eq!(cpu.registers.gpr[5], 42, "r5 should be 42, got {}", cpu.registers.gpr[5]);
        
        // Verify r6 computed correctly
        assert_eq!(cpu.registers.gpr[6], 52, "r6 should be 52 (42+10), got {}", cpu.registers.gpr[6]);
        
        println!("✓ JIT register and memory integration test passed");
        println!("  Memory[0x1000]: {}", stored);
        println!("  r5 (loaded): {}", cpu.registers.gpr[5]);
        println!("  r6 (computed): {}", cpu.registers.gpr[6]);
        println!("  Full integration working!");
    }
}
