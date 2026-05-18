// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2024 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Tests for floating-point instructions

#[cfg(test)]
mod tests {
    use newton_cpu::{Cpu, PpcModel, Registers};

    #[test]
    fn test_fadd_double_precision() {
        let mut cpu = Cpu::new(PpcModel::G4);
        let regs = &mut cpu.registers;
        
        // Set up test values
        regs.fpr[1] = 2.5;
        regs.fpr[2] = 3.7;
        
        // Execute fadd f3, f1, f2
        newton_cpu::interpreter::Interpreter::new()
            .execute(
                newton_cpu::decoder::Instruction::Fadd { frt: 3, fra: 1, frb: 2, rc: false },
                regs
            )
            .unwrap();
        
        // Check result
        assert_eq!(regs.fpr[3], 6.2);
    }

    #[test]
    fn test_fadds_single_precision() {
        let mut cpu = Cpu::new(PpcModel::G4);
        let regs = &mut cpu.registers;
        
        // Set up test values
        regs.fpr[1] = 1.5;
        regs.fpr[2] = 2.25;
        
        // Execute fadds f3, f1, f2
        newton_cpu::interpreter::Interpreter::new()
            .execute(
                newton_cpu::decoder::Instruction::Fadds { frt: 3, fra: 1, frb: 2, rc: false },
                regs
            )
            .unwrap();
        
        // Check result (should be stored as double but computed as single)
        let result = regs.fpr[3];
        assert!((result - 3.75).abs() < 0.0001);
    }

    #[test]
    fn test_fsub() {
        let mut cpu = Cpu::new(PpcModel::G4);
        let regs = &mut cpu.registers;
        
        regs.fpr[1] = 10.0;
        regs.fpr[2] = 3.5;
        
        // Execute fsub f3, f1, f2
        newton_cpu::interpreter::Interpreter::new()
            .execute(
                newton_cpu::decoder::Instruction::Fsub { frt: 3, fra: 1, frb: 2, rc: false },
                regs
            )
            .unwrap();
        
        assert_eq!(regs.fpr[3], 6.5);
    }

    #[test]
    fn test_fmul() {
        let mut cpu = Cpu::new(PpcModel::G4);
        let regs = &mut cpu.registers;
        
        regs.fpr[1] = 2.5;
        regs.fpr[2] = 4.0;
        
        // Execute fmul f3, f1, f2
        newton_cpu::interpreter::Interpreter::new()
            .execute(
                newton_cpu::decoder::Instruction::Fmul { frt: 3, fra: 1, frc: 2, rc: false },
                regs
            )
            .unwrap();
        
        assert_eq!(regs.fpr[3], 10.0);
    }

    #[test]
    fn test_fdiv() {
        let mut cpu = Cpu::new(PpcModel::G4);
        let regs = &mut cpu.registers;
        
        regs.fpr[1] = 10.0;
        regs.fpr[2] = 2.5;
        
        // Execute fdiv f3, f1, f2
        newton_cpu::interpreter::Interpreter::new()
            .execute(
                newton_cpu::decoder::Instruction::Fdiv { frt: 3, fra: 1, frb: 2, rc: false },
                regs
            )
            .unwrap();
        
        assert_eq!(regs.fpr[3], 4.0);
    }

    #[test]
    fn test_fdiv_by_zero() {
        let mut cpu = Cpu::new(PpcModel::G4);
        let regs = &mut cpu.registers;
        
        regs.fpr[1] = 1.0;
        regs.fpr[2] = 0.0;
        
        // Execute fdiv f3, f1, f2
        newton_cpu::interpreter::Interpreter::new()
            .execute(
                newton_cpu::decoder::Instruction::Fdiv { frt: 3, fra: 1, frb: 2, rc: false },
                regs
            )
            .unwrap();
        
        // Result should be infinity
        assert!(regs.fpr[3].is_infinite());
    }

    #[test]
    fn test_fp_operations_chain() {
        let mut cpu = Cpu::new(PpcModel::G4);
        let regs = &mut cpu.registers;
        
        // Set up: f1 = 5.0, f2 = 3.0, f3 = 2.0
        regs.fpr[1] = 5.0;
        regs.fpr[2] = 3.0;
        regs.fpr[3] = 2.0;
        
        let mut interp = newton_cpu::interpreter::Interpreter::new();
        
        // f4 = f1 + f2  (5.0 + 3.0 = 8.0)
        interp.execute(
            newton_cpu::decoder::Instruction::Fadd { frt: 4, fra: 1, frb: 2, rc: false },
            regs
        ).unwrap();
        assert_eq!(regs.fpr[4], 8.0);
        
        // f5 = f4 * f3  (8.0 * 2.0 = 16.0)
        interp.execute(
            newton_cpu::decoder::Instruction::Fmul { frt: 5, fra: 4, frc: 3, rc: false },
            regs
        ).unwrap();
        assert_eq!(regs.fpr[5], 16.0);
        
        // f6 = f5 / f3  (16.0 / 2.0 = 8.0)
        interp.execute(
            newton_cpu::decoder::Instruction::Fdiv { frt: 6, fra: 5, frb: 3, rc: false },
            regs
        ).unwrap();
        assert_eq!(regs.fpr[6], 8.0);
        
        // f7 = f6 - f2  (8.0 - 3.0 = 5.0)
        interp.execute(
            newton_cpu::decoder::Instruction::Fsub { frt: 7, fra: 6, frb: 2, rc: false },
            regs
        ).unwrap();
        assert_eq!(regs.fpr[7], 5.0);
    }

    #[test]
    fn test_negative_numbers() {
        let mut cpu = Cpu::new(PpcModel::G4);
        let regs = &mut cpu.registers;
        
        regs.fpr[1] = -2.5;
        regs.fpr[2] = 3.5;
        
        // Test addition with negative
        newton_cpu::interpreter::Interpreter::new()
            .execute(
                newton_cpu::decoder::Instruction::Fadd { frt: 3, fra: 1, frb: 2, rc: false },
                regs
            )
            .unwrap();
        assert_eq!(regs.fpr[3], 1.0);
        
        // Test multiplication with negative
        newton_cpu::interpreter::Interpreter::new()
            .execute(
                newton_cpu::decoder::Instruction::Fmul { frt: 4, fra: 1, frc: 2, rc: false },
                regs
            )
            .unwrap();
        assert_eq!(regs.fpr[4], -8.75);
    }
}
