// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
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

    #[test]
    fn test_fneg() {
        let mut cpu = Cpu::new(PpcModel::G4);
        let regs = &mut cpu.registers;
        
        regs.fpr[1] = 5.5;
        
        newton_cpu::interpreter::Interpreter::new()
            .execute(
                newton_cpu::decoder::Instruction::Fneg { frt: 2, frb: 1, rc: false },
                regs
            )
            .unwrap();
        
        assert_eq!(regs.fpr[2], -5.5);
    }

    #[test]
    fn test_fabs() {
        let mut cpu = Cpu::new(PpcModel::G4);
        let regs = &mut cpu.registers;
        
        regs.fpr[1] = -7.3;
        
        newton_cpu::interpreter::Interpreter::new()
            .execute(
                newton_cpu::decoder::Instruction::Fabs { frt: 2, frb: 1, rc: false },
                regs
            )
            .unwrap();
        
        assert_eq!(regs.fpr[2], 7.3);
    }

    #[test]
    fn test_fmr() {
        let mut cpu = Cpu::new(PpcModel::G4);
        let regs = &mut cpu.registers;
        
        regs.fpr[1] = 3.14159;
        
        newton_cpu::interpreter::Interpreter::new()
            .execute(
                newton_cpu::decoder::Instruction::Fmr { frt: 2, frb: 1, rc: false },
                regs
            )
            .unwrap();
        
        assert_eq!(regs.fpr[2], 3.14159);
    }

    #[test]
    fn test_fsqrt() {
        let mut cpu = Cpu::new(PpcModel::G4);
        let regs = &mut cpu.registers;
        
        regs.fpr[1] = 16.0;
        
        newton_cpu::interpreter::Interpreter::new()
            .execute(
                newton_cpu::decoder::Instruction::Fsqrt { frt: 2, frb: 1, rc: false },
                regs
            )
            .unwrap();
        
        assert_eq!(regs.fpr[2], 4.0);
    }

    #[test]
    fn test_fsqrts() {
        let mut cpu = Cpu::new(PpcModel::G4);
        let regs = &mut cpu.registers;
        
        regs.fpr[1] = 25.0;
        
        newton_cpu::interpreter::Interpreter::new()
            .execute(
                newton_cpu::decoder::Instruction::Fsqrts { frt: 2, frb: 1, rc: false },
                regs
            )
            .unwrap();
        
        assert_eq!(regs.fpr[2], 5.0);
    }

    #[test]
    fn test_fmadd() {
        let mut cpu = Cpu::new(PpcModel::G4);
        let regs = &mut cpu.registers;
        
        // fmadd: frt = (fra * frc) + frb
        regs.fpr[1] = 2.0;  // fra
        regs.fpr[2] = 3.0;  // frc
        regs.fpr[3] = 4.0;  // frb
        
        newton_cpu::interpreter::Interpreter::new()
            .execute(
                newton_cpu::decoder::Instruction::Fmadd { frt: 4, fra: 1, frc: 2, frb: 3, rc: false },
                regs
            )
            .unwrap();
        
        // (2.0 * 3.0) + 4.0 = 10.0
        assert_eq!(regs.fpr[4], 10.0);
    }

    #[test]
    fn test_fmadds() {
        let mut cpu = Cpu::new(PpcModel::G4);
        let regs = &mut cpu.registers;
        
        regs.fpr[1] = 1.5;
        regs.fpr[2] = 2.0;
        regs.fpr[3] = 0.5;
        
        newton_cpu::interpreter::Interpreter::new()
            .execute(
                newton_cpu::decoder::Instruction::Fmadds { frt: 4, fra: 1, frc: 2, frb: 3, rc: false },
                regs
            )
            .unwrap();
        
        // (1.5 * 2.0) + 0.5 = 3.5
        assert_eq!(regs.fpr[4], 3.5);
    }

    #[test]
    fn test_fmsub() {
        let mut cpu = Cpu::new(PpcModel::G4);
        let regs = &mut cpu.registers;
        
        // fmsub: frt = (fra * frc) - frb
        regs.fpr[1] = 5.0;
        regs.fpr[2] = 2.0;
        regs.fpr[3] = 3.0;
        
        newton_cpu::interpreter::Interpreter::new()
            .execute(
                newton_cpu::decoder::Instruction::Fmsub { frt: 4, fra: 1, frc: 2, frb: 3, rc: false },
                regs
            )
            .unwrap();
        
        // (5.0 * 2.0) - 3.0 = 7.0
        assert_eq!(regs.fpr[4], 7.0);
    }

    #[test]
    fn test_fnmadd() {
        let mut cpu = Cpu::new(PpcModel::G4);
        let regs = &mut cpu.registers;
        
        // fnmadd: frt = -((fra * frc) + frb)
        regs.fpr[1] = 2.0;
        regs.fpr[2] = 3.0;
        regs.fpr[3] = 4.0;
        
        newton_cpu::interpreter::Interpreter::new()
            .execute(
                newton_cpu::decoder::Instruction::Fnmadd { frt: 4, fra: 1, frc: 2, frb: 3, rc: false },
                regs
            )
            .unwrap();
        
        // -((2.0 * 3.0) + 4.0) = -10.0
        assert_eq!(regs.fpr[4], -10.0);
    }

    #[test]
    fn test_fnmsub() {
        let mut cpu = Cpu::new(PpcModel::G4);
        let regs = &mut cpu.registers;
        
        // fnmsub: frt = -((fra * frc) - frb)
        regs.fpr[1] = 5.0;
        regs.fpr[2] = 2.0;
        regs.fpr[3] = 3.0;
        
        newton_cpu::interpreter::Interpreter::new()
            .execute(
                newton_cpu::decoder::Instruction::Fnmsub { frt: 4, fra: 1, frc: 2, frb: 3, rc: false },
                regs
            )
            .unwrap();
        
        // -((5.0 * 2.0) - 3.0) = -7.0
        assert_eq!(regs.fpr[4], -7.0);
    }

    #[test]
    fn test_fused_multiply_add_precision() {
        let mut cpu = Cpu::new(PpcModel::G4);
        let regs = &mut cpu.registers;
        
        // Test that fused multiply-add maintains better precision
        regs.fpr[1] = 1e10;
        regs.fpr[2] = 1e-10;
        regs.fpr[3] = 1.0;
        
        newton_cpu::interpreter::Interpreter::new()
            .execute(
                newton_cpu::decoder::Instruction::Fmadd { frt: 4, fra: 1, frc: 2, frb: 3, rc: false },
                regs
            )
            .unwrap();
        
        // Result should be 2.0 (1.0 + 1.0)
        assert_eq!(regs.fpr[4], 2.0);
    }

    #[test]
    fn test_fcmpu_equal() {
        let mut cpu = Cpu::new(PpcModel::G4);
        let regs = &mut cpu.registers;
        
        regs.fpr[1] = 5.0;
        regs.fpr[2] = 5.0;
        
        newton_cpu::interpreter::Interpreter::new()
            .execute(
                newton_cpu::decoder::Instruction::Fcmpu { crfd: 0, fra: 1, frb: 2 },
                regs
            )
            .unwrap();
        
        // CR0 should have EQ bit set (0x2 in bits 28-31)
        let cr0 = (regs.cr.bits() >> 28) & 0xF;
        assert_eq!(cr0, 0x2); // EQ
    }

    #[test]
    fn test_fcmpu_less_than() {
        let mut cpu = Cpu::new(PpcModel::G4);
        let regs = &mut cpu.registers;
        
        regs.fpr[1] = 3.0;
        regs.fpr[2] = 5.0;
        
        newton_cpu::interpreter::Interpreter::new()
            .execute(
                newton_cpu::decoder::Instruction::Fcmpu { crfd: 0, fra: 1, frb: 2 },
                regs
            )
            .unwrap();
        
        // CR0 should have LT bit set (0x8)
        let cr0 = (regs.cr.bits() >> 28) & 0xF;
        assert_eq!(cr0, 0x8); // LT
    }

    #[test]
    fn test_fcmpu_greater_than() {
        let mut cpu = Cpu::new(PpcModel::G4);
        let regs = &mut cpu.registers;
        
        regs.fpr[1] = 10.0;
        regs.fpr[2] = 5.0;
        
        newton_cpu::interpreter::Interpreter::new()
            .execute(
                newton_cpu::decoder::Instruction::Fcmpu { crfd: 0, fra: 1, frb: 2 },
                regs
            )
            .unwrap();
        
        // CR0 should have GT bit set (0x4)
        let cr0 = (regs.cr.bits() >> 28) & 0xF;
        assert_eq!(cr0, 0x4); // GT
    }

    #[test]
    fn test_fcmpu_unordered() {
        let mut cpu = Cpu::new(PpcModel::G4);
        let regs = &mut cpu.registers;
        
        regs.fpr[1] = f64::NAN;
        regs.fpr[2] = 5.0;
        
        newton_cpu::interpreter::Interpreter::new()
            .execute(
                newton_cpu::decoder::Instruction::Fcmpu { crfd: 0, fra: 1, frb: 2 },
                regs
            )
            .unwrap();
        
        // CR0 should have FU (unordered) bit set (0x1)
        let cr0 = (regs.cr.bits() >> 28) & 0xF;
        assert_eq!(cr0, 0x1); // FU
    }

    #[test]
    fn test_fcmpu_different_cr_field() {
        let mut cpu = Cpu::new(PpcModel::G4);
        let regs = &mut cpu.registers;
        
        regs.fpr[1] = 7.0;
        regs.fpr[2] = 3.0;
        
        // Compare into CR1 instead of CR0
        newton_cpu::interpreter::Interpreter::new()
            .execute(
                newton_cpu::decoder::Instruction::Fcmpu { crfd: 1, fra: 1, frb: 2 },
                regs
            )
            .unwrap();
        
        // CR1 should have GT bit set (0x4)
        let cr1 = (regs.cr.bits() >> 24) & 0xF;
        assert_eq!(cr1, 0x4); // GT
    }

    #[test]
    fn test_fcmpo() {
        let mut cpu = Cpu::new(PpcModel::G4);
        let regs = &mut cpu.registers;
        
        regs.fpr[1] = 3.5;
        regs.fpr[2] = 7.2;
        
        newton_cpu::interpreter::Interpreter::new()
            .execute(
                newton_cpu::decoder::Instruction::Fcmpo { crfd: 0, fra: 1, frb: 2 },
                regs
            )
            .unwrap();
        
        // Should be LT
        let cr0 = (regs.cr.bits() >> 28) & 0xF;
        assert_eq!(cr0, 0x8); // LT
    }

    #[test]
    fn test_fctiwz_positive() {
        let mut cpu = Cpu::new(PpcModel::G4);
        let regs = &mut cpu.registers;
        
        regs.fpr[1] = 42.7;
        
        newton_cpu::interpreter::Interpreter::new()
            .execute(
                newton_cpu::decoder::Instruction::Fctiwz { frt: 2, frb: 1, rc: false },
                regs
            )
            .unwrap();
        
        // Extract the lower 32 bits as integer
        let bits = regs.fpr[2].to_bits();
        let int_result = (bits & 0xFFFF_FFFF) as i32;
        assert_eq!(int_result, 42);
    }

    #[test]
    fn test_fctiwz_negative() {
        let mut cpu = Cpu::new(PpcModel::G4);
        let regs = &mut cpu.registers;
        
        regs.fpr[1] = -17.9;
        
        newton_cpu::interpreter::Interpreter::new()
            .execute(
                newton_cpu::decoder::Instruction::Fctiwz { frt: 2, frb: 1, rc: false },
                regs
            )
            .unwrap();
        
        // Extract the lower 32 bits as integer
        let bits = regs.fpr[2].to_bits();
        let int_result = (bits & 0xFFFF_FFFF) as i32;
        assert_eq!(int_result, -17);
    }

    #[test]
    fn test_fctiwz_truncation() {
        let mut cpu = Cpu::new(PpcModel::G4);
        let regs = &mut cpu.registers;
        
        // Test that it truncates (rounds toward zero)
        regs.fpr[1] = 3.9;
        
        newton_cpu::interpreter::Interpreter::new()
            .execute(
                newton_cpu::decoder::Instruction::Fctiwz { frt: 2, frb: 1, rc: false },
                regs
            )
            .unwrap();
        
        let bits = regs.fpr[2].to_bits();
        let int_result = (bits & 0xFFFF_FFFF) as i32;
        assert_eq!(int_result, 3);
        
        // Test negative truncation
        regs.fpr[1] = -3.9;
        newton_cpu::interpreter::Interpreter::new()
            .execute(
                newton_cpu::decoder::Instruction::Fctiwz { frt: 2, frb: 1, rc: false },
                regs
            )
            .unwrap();
        
        let bits = regs.fpr[2].to_bits();
        let int_result = (bits & 0xFFFF_FFFF) as i32;
        assert_eq!(int_result, -3);
    }

    #[test]
    fn test_frsp() {
        let mut cpu = Cpu::new(PpcModel::G4);
        let regs = &mut cpu.registers;
        
        // Use a value that has different representations in f32 vs f64
        regs.fpr[1] = 1.0 / 3.0;
        
        newton_cpu::interpreter::Interpreter::new()
            .execute(
                newton_cpu::decoder::Instruction::Frsp { frt: 2, frb: 1, rc: false },
                regs
            )
            .unwrap();
        
        // Result should be rounded to single precision
        let expected = (1.0f32 / 3.0f32) as f64;
        assert_eq!(regs.fpr[2], expected);
    }

    #[test]
    fn test_frsp_preserves_simple_values() {
        let mut cpu = Cpu::new(PpcModel::G4);
        let regs = &mut cpu.registers;
        
        // Simple values should be preserved
        regs.fpr[1] = 5.0;
        
        newton_cpu::interpreter::Interpreter::new()
            .execute(
                newton_cpu::decoder::Instruction::Frsp { frt: 2, frb: 1, rc: false },
                regs
            )
            .unwrap();
        
        assert_eq!(regs.fpr[2], 5.0);
    }
}
