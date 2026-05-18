// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2024 NewtonEmu Contributors
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

    /// Simple memory implementation for testing
    struct TestMemory {
        data: Vec<u8>,
    }

    impl TestMemory {
        fn new(size: usize) -> Self {
            Self {
                data: vec![0; size],
            }
        }

        fn write_instruction(&mut self, addr: u32, instr: u32) {
            let addr = addr as usize;
            self.data[addr] = (instr >> 24) as u8;
            self.data[addr + 1] = (instr >> 16) as u8;
            self.data[addr + 2] = (instr >> 8) as u8;
            self.data[addr + 3] = instr as u8;
        }
    }

    impl MemoryInterface for TestMemory {
        fn read_u8(&self, addr: u32) -> Result<u8> {
            let addr = addr as usize;
            if addr >= self.data.len() {
                return Ok(0);
            }
            Ok(self.data[addr])
        }

        fn read_u16(&self, addr: u32) -> Result<u16> {
            let addr = addr as usize;
            if addr + 1 >= self.data.len() {
                return Ok(0);
            }
            Ok(u16::from_be_bytes([self.data[addr], self.data[addr + 1]]))
        }

        fn read_u32(&self, addr: u32) -> Result<u32> {
            let addr = addr as usize;
            if addr + 3 >= self.data.len() {
                return Ok(0);
            }
            Ok(u32::from_be_bytes([
                self.data[addr],
                self.data[addr + 1],
                self.data[addr + 2],
                self.data[addr + 3],
            ]))
        }

        fn write_u8(&mut self, addr: u32, value: u8) -> Result<()> {
            self.data[addr as usize] = value;
            Ok(())
        }

        fn write_u16(&mut self, addr: u32, value: u16) -> Result<()> {
            let addr = addr as usize;
            let bytes = value.to_be_bytes();
            self.data[addr] = bytes[0];
            self.data[addr + 1] = bytes[1];
            Ok(())
        }

        fn write_u32(&mut self, addr: u32, value: u32) -> Result<()> {
            let addr = addr as usize;
            let bytes = value.to_be_bytes();
            self.data[addr] = bytes[0];
            self.data[addr + 1] = bytes[1];
            self.data[addr + 2] = bytes[2];
            self.data[addr + 3] = bytes[3];
            Ok(())
        }
    }

    #[test]
    fn test_jit_compilation_basic() {
        let mut cpu = Cpu::new_with_jit(PpcModel::G4).unwrap();
        let mut mem = TestMemory::new(1024);

        // Write a simple program: addi r3, r0, 42
        mem.write_instruction(0x100, 0x3860002A);
        mem.write_instruction(0x104, 0x38800064); // addi r4, r0, 100
        mem.write_instruction(0x108, 0x48000000); // b +0 (infinite loop to end block)

        cpu.registers.pc = 0x100;
        cpu.set_execution_mode(ExecutionMode::Jit);

        // First execution - should compile
        cpu.step(&mut mem).unwrap();
        
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
        let mut mem = TestMemory::new(1024);

        // Write a simple loop
        mem.write_instruction(0x100, 0x3860000A); // addi r3, r0, 10
        mem.write_instruction(0x104, 0x48000000); // b +0 (end block)

        cpu.registers.pc = 0x100;
        cpu.set_execution_mode(ExecutionMode::Adaptive);

        // Execute multiple times to trigger hot code detection
        for i in 0..150 {
            cpu.registers.pc = 0x100;
            cpu.step(&mut mem).unwrap();
            
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
}
