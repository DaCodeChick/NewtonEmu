// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
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
            Ok(self.data.borrow()[addr as usize])
        }

        fn read_u16(&self, addr: u32) -> Result<u16> {
            let addr = addr as usize;
            let data = self.data.borrow();
            Ok(u16::from_be_bytes([data[addr], data[addr + 1]]))
        }

        fn read_u32(&self, addr: u32) -> Result<u32> {
            let addr = addr as usize;
            let data = self.data.borrow();
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
    fn test_normal_instruction_advances_pc() {
        let mut cpu = Cpu::new(PpcModel::G4);
        let mem = TestMemory::new(1024);

        // addi r3, r0, 100  (0x38600064)
        mem.write_instruction(0x100, 0x38600064);

        cpu.registers.pc = 0x100;
        cpu.step(&mem).unwrap();

        // PC should advance by 4
        assert_eq!(cpu.registers.pc, 0x104);
        // r3 should be 100
        assert_eq!(cpu.registers.gpr[3], 100);
    }

    #[test]
    fn test_unconditional_branch_modifies_pc() {
        let mut cpu = Cpu::new(PpcModel::G4);
        let mem = TestMemory::new(1024);

        // b 0x200 (relative branch, li = 0x100 >> 2 = 0x40)
        // Opcode: 18 | (li << 2) | aa | lk
        // b +0x100: 0x48000100
        mem.write_instruction(0x100, 0x48000100);

        cpu.registers.pc = 0x100;
        cpu.step(&mem).unwrap();

        // PC should be 0x100 + 0x100 = 0x200
        assert_eq!(cpu.registers.pc, 0x200, "Branch should set PC to 0x200");
    }

    #[test]
    fn test_conditional_branch_taken() {
        let mut cpu = Cpu::new(PpcModel::G4);
        let mem = TestMemory::new(1024);

        // Set up condition: CR0[EQ] = 1
        cpu.registers.cr.insert(newton_cpu::registers::ConditionRegister::CR0_EQ);

        // beq +0x10 (branch if equal)
        // bc with BO=12 (0b01100 - branch if CR[BI]=1), BI=2 (CR0[EQ]), BD=0x10
        // Opcode: 0x4182_0010
        mem.write_instruction(0x100, 0x41820010);

        cpu.registers.pc = 0x100;
        cpu.step(&mem).unwrap();

        // PC should be 0x110 (0x100 + 0x10)
        assert_eq!(cpu.registers.pc, 0x110, "Conditional branch should be taken");
    }

    #[test]
    fn test_conditional_branch_not_taken() {
        let mut cpu = Cpu::new(PpcModel::G4);
        let mem = TestMemory::new(1024);

        // Clear condition: CR0[EQ] = 0
        cpu.registers.cr.remove(newton_cpu::registers::ConditionRegister::CR0_EQ);

        // beq +0x10 (branch if equal)
        mem.write_instruction(0x100, 0x41820010);

        cpu.registers.pc = 0x100;
        cpu.step(&mem).unwrap();

        // PC should advance by 4 (branch not taken)
        assert_eq!(cpu.registers.pc, 0x104, "Conditional branch should not be taken");
    }

    #[test]
    fn test_branch_with_link() {
        let mut cpu = Cpu::new(PpcModel::G4);
        let mem = TestMemory::new(1024);

        // bl +0x100 (branch and link)
        // Opcode: 0x48000101
        mem.write_instruction(0x100, 0x48000101);

        cpu.registers.pc = 0x100;
        cpu.step(&mem).unwrap();

        // PC should be 0x200
        assert_eq!(cpu.registers.pc, 0x200);
        // LR should be 0x104 (return address)
        assert_eq!(cpu.registers.lr, 0x104);
    }

    #[test]
    fn test_arithmetic_sequence() {
        let mut cpu = Cpu::new(PpcModel::G4);
        let mem = TestMemory::new(1024);

        // addi r3, r0, 10  (0x3860000A)
        mem.write_instruction(0x100, 0x3860000A);
        // addi r4, r0, 20  (0x38800014)
        mem.write_instruction(0x104, 0x38800014);
        // add r5, r3, r4   (0x7CA32214)
        mem.write_instruction(0x108, 0x7CA32214);

        cpu.registers.pc = 0x100;

        // Execute first instruction
        cpu.step(&mem).unwrap();
        assert_eq!(cpu.registers.pc, 0x104);
        assert_eq!(cpu.registers.gpr[3], 10);

        // Execute second instruction
        cpu.step(&mem).unwrap();
        assert_eq!(cpu.registers.pc, 0x108);
        assert_eq!(cpu.registers.gpr[4], 20);

        // Execute third instruction
        cpu.step(&mem).unwrap();
        assert_eq!(cpu.registers.pc, 0x10C);
        assert_eq!(cpu.registers.gpr[5], 30);
    }

    #[test]
    fn test_spr_instructions() {
        let mut cpu = Cpu::new(PpcModel::G4);
        let mem = TestMemory::new(1024);

        // mtspr LR, r3  (SPR 8 = LR)
        // 0x7C6803A6 = mtspr 8, r3
        mem.write_instruction(0x100, 0x7C6803A6);
        
        // mtspr CTR, r4  (SPR 9 = CTR)
        // 0x7C8903A6 = mtspr 9, r4
        mem.write_instruction(0x104, 0x7C8903A6);
        
        // mfspr r5, LR  (SPR 8 = LR)
        // 0x7CA802A6 = mfspr r5, 8
        mem.write_instruction(0x108, 0x7CA802A6);
        
        // mfspr r6, CTR  (SPR 9 = CTR)
        // 0x7CC902A6 = mfspr r6, 9
        mem.write_instruction(0x10C, 0x7CC902A6);
        
        cpu.registers.gpr[3] = 0x1234_5678;
        cpu.registers.gpr[4] = 0xABCD_EF00;
        cpu.registers.pc = 0x100;

        // mtspr LR, r3
        cpu.step(&mem).unwrap();
        assert_eq!(cpu.registers.lr, 0x1234_5678);
        assert_eq!(cpu.registers.pc, 0x104);

        // mtspr CTR, r4
        cpu.step(&mem).unwrap();
        assert_eq!(cpu.registers.ctr, 0xABCD_EF00);
        assert_eq!(cpu.registers.pc, 0x108);

        // mfspr r5, LR
        cpu.step(&mem).unwrap();
        assert_eq!(cpu.registers.gpr[5], 0x1234_5678);
        assert_eq!(cpu.registers.pc, 0x10C);

        // mfspr r6, CTR
        cpu.step(&mem).unwrap();
        assert_eq!(cpu.registers.gpr[6], 0xABCD_EF00);
        assert_eq!(cpu.registers.pc, 0x110);
    }

    #[test]
    fn test_msr_instructions() {
        let mut cpu = Cpu::new(PpcModel::G4);
        let mem = TestMemory::new(1024);

        // mtmsr r3
        // 0x7C600124 = mtmsr r3
        mem.write_instruction(0x100, 0x7C600124);
        
        // mfmsr r4
        // 0x7C8000A6 = mfmsr r4
        mem.write_instruction(0x104, 0x7C8000A6);
        
        cpu.registers.gpr[3] = 0x0000_9032; // Set some MSR bits
        cpu.registers.pc = 0x100;

        // mtmsr r3
        cpu.step(&mem).unwrap();
        assert_eq!(cpu.registers.msr.bits(), 0x0000_9032);
        assert_eq!(cpu.registers.pc, 0x104);

        // mfmsr r4
        cpu.step(&mem).unwrap();
        assert_eq!(cpu.registers.gpr[4], 0x0000_9032);
        assert_eq!(cpu.registers.pc, 0x108);
    }

    #[test]
    fn test_cr_instructions() {
        let mut cpu = Cpu::new(PpcModel::G4);
        let mem = TestMemory::new(1024);

        // mtcrf 0xFF, r3  (move all CR fields)
        // 0x7C6FF120 = mtcrf 0xFF, r3
        mem.write_instruction(0x100, 0x7C6FF120);
        
        // mfcr r4
        // 0x7C800026 = mfcr r4
        mem.write_instruction(0x104, 0x7C800026);
        
        cpu.registers.gpr[3] = 0x8421_0842;
        cpu.registers.pc = 0x100;

        // mtcrf 0xFF, r3
        cpu.step(&mem).unwrap();
        assert_eq!(cpu.registers.cr.bits(), 0x8421_0842);
        assert_eq!(cpu.registers.pc, 0x104);

        // mfcr r4
        cpu.step(&mem).unwrap();
        assert_eq!(cpu.registers.gpr[4], 0x8421_0842);
        assert_eq!(cpu.registers.pc, 0x108);
    }

    #[test]
    fn test_pvr_readonly() {
        let mut cpu = Cpu::new(PpcModel::G4);
        let mem = TestMemory::new(1024);

        // mfspr r3, PVR  (SPR 287 = PVR)
        // PVR = 287 = 0x11F, spr_low=31, spr_high=8
        // 0x7C7F42A6 = mfspr r3, 287
        mem.write_instruction(0x100, 0x7C7F42A6);
        
        cpu.registers.pc = 0x100;

        // mfspr r3, PVR
        cpu.step(&mem).unwrap();
        // G4 (7400) PVR
        assert_eq!(cpu.registers.gpr[3], 0x800C_1101);
        assert_eq!(cpu.registers.pc, 0x104);
    }
}
