// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2024 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Tests for CPU execution and PC tracking

#[cfg(test)]
mod tests {
    use newton_cpu::{Cpu, PpcModel, MemoryInterface};
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
            Ok(self.data[addr as usize])
        }

        fn read_u16(&self, addr: u32) -> Result<u16> {
            let addr = addr as usize;
            Ok(u16::from_be_bytes([self.data[addr], self.data[addr + 1]]))
        }

        fn read_u32(&self, addr: u32) -> Result<u32> {
            let addr = addr as usize;
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
    fn test_normal_instruction_advances_pc() {
        let mut cpu = Cpu::new(PpcModel::G4);
        let mut mem = TestMemory::new(1024);

        // addi r3, r0, 100  (0x38600064)
        mem.write_instruction(0x100, 0x38600064);

        cpu.registers.pc = 0x100;
        cpu.step(&mut mem).unwrap();

        // PC should advance by 4
        assert_eq!(cpu.registers.pc, 0x104);
        // r3 should be 100
        assert_eq!(cpu.registers.gpr[3], 100);
    }

    #[test]
    fn test_unconditional_branch_modifies_pc() {
        let mut cpu = Cpu::new(PpcModel::G4);
        let mut mem = TestMemory::new(1024);

        // b 0x200 (relative branch, li = 0x100 >> 2 = 0x40)
        // Opcode: 18 | (li << 2) | aa | lk
        // b +0x100: 0x48000100
        mem.write_instruction(0x100, 0x48000100);

        cpu.registers.pc = 0x100;
        cpu.step(&mut mem).unwrap();

        // PC should be 0x100 + 0x100 = 0x200
        assert_eq!(cpu.registers.pc, 0x200, "Branch should set PC to 0x200");
    }

    #[test]
    fn test_conditional_branch_taken() {
        let mut cpu = Cpu::new(PpcModel::G4);
        let mut mem = TestMemory::new(1024);

        // Set up condition: CR0[EQ] = 1
        cpu.registers.cr.insert(newton_cpu::registers::ConditionRegister::CR0_EQ);

        // beq +0x10 (branch if equal)
        // bc with BO=12 (0b01100 - branch if CR[BI]=1), BI=2 (CR0[EQ]), BD=0x10
        // Opcode: 0x4182_0010
        mem.write_instruction(0x100, 0x41820010);

        cpu.registers.pc = 0x100;
        cpu.step(&mut mem).unwrap();

        // PC should be 0x110 (0x100 + 0x10)
        assert_eq!(cpu.registers.pc, 0x110, "Conditional branch should be taken");
    }

    #[test]
    fn test_conditional_branch_not_taken() {
        let mut cpu = Cpu::new(PpcModel::G4);
        let mut mem = TestMemory::new(1024);

        // Clear condition: CR0[EQ] = 0
        cpu.registers.cr.remove(newton_cpu::registers::ConditionRegister::CR0_EQ);

        // beq +0x10 (branch if equal)
        mem.write_instruction(0x100, 0x41820010);

        cpu.registers.pc = 0x100;
        cpu.step(&mut mem).unwrap();

        // PC should advance by 4 (branch not taken)
        assert_eq!(cpu.registers.pc, 0x104, "Conditional branch should not be taken");
    }

    #[test]
    fn test_branch_with_link() {
        let mut cpu = Cpu::new(PpcModel::G4);
        let mut mem = TestMemory::new(1024);

        // bl +0x100 (branch and link)
        // Opcode: 0x48000101
        mem.write_instruction(0x100, 0x48000101);

        cpu.registers.pc = 0x100;
        cpu.step(&mut mem).unwrap();

        // PC should be 0x200
        assert_eq!(cpu.registers.pc, 0x200);
        // LR should be 0x104 (return address)
        assert_eq!(cpu.registers.lr, 0x104);
    }

    #[test]
    fn test_arithmetic_sequence() {
        let mut cpu = Cpu::new(PpcModel::G4);
        let mut mem = TestMemory::new(1024);

        // addi r3, r0, 10  (0x3860000A)
        mem.write_instruction(0x100, 0x3860000A);
        // addi r4, r0, 20  (0x38800014)
        mem.write_instruction(0x104, 0x38800014);
        // add r5, r3, r4   (0x7CA32214)
        mem.write_instruction(0x108, 0x7CA32214);

        cpu.registers.pc = 0x100;

        // Execute first instruction
        cpu.step(&mut mem).unwrap();
        assert_eq!(cpu.registers.pc, 0x104);
        assert_eq!(cpu.registers.gpr[3], 10);

        // Execute second instruction
        cpu.step(&mut mem).unwrap();
        assert_eq!(cpu.registers.pc, 0x108);
        assert_eq!(cpu.registers.gpr[4], 20);

        // Execute third instruction
        cpu.step(&mut mem).unwrap();
        assert_eq!(cpu.registers.pc, 0x10C);
        assert_eq!(cpu.registers.gpr[5], 30);
    }
}
