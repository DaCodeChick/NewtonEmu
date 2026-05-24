// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! 68k instruction decoding

use crate::registers::ConditionCode;

/// Operand size for 68k instructions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Size {
    Byte,   // 8-bit
    Word,   // 16-bit
    Long,   // 32-bit
}

impl Size {
    /// Get size in bytes
    pub fn bytes(&self) -> u32 {
        match self {
            Size::Byte => 1,
            Size::Word => 2,
            Size::Long => 4,
        }
    }
    
    /// Decode from instruction size bits
    pub fn from_bits(bits: u8) -> Option<Self> {
        match bits & 0x03 {
            0b00 => Some(Size::Byte),
            0b01 => Some(Size::Word),
            0b10 => Some(Size::Long),
            _ => None,
        }
    }
}

/// Addressing modes for 68k
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressingMode {
    /// Dn - Data register direct
    DataRegister(u8),
    /// An - Address register direct
    AddressRegister(u8),
    /// (An) - Address register indirect
    AddressIndirect(u8),
    /// (An)+ - Address register indirect with postincrement
    AddressPostIncrement(u8),
    /// -(An) - Address register indirect with predecrement
    AddressPreDecrement(u8),
    /// d(An) - Address register indirect with displacement
    AddressDisplacement(u8, i16),
    /// d(An,Xn) - Address register indirect with index
    AddressIndex(u8, u8, Size, i8),
    /// xxx.W - Absolute short address
    AbsoluteShort(i16),
    /// xxx.L - Absolute long address
    AbsoluteLong(u32),
    /// d(PC) - PC-relative with displacement
    PcDisplacement(i16),
    /// d(PC,Xn) - PC-relative with index
    PcIndex(u8, Size, i8),
    /// #xxx - Immediate data
    Immediate(u32),
}

/// Decoded 68k instruction
#[derive(Debug, Clone)]
pub enum Instruction {
    // Data movement
    Move {
        size: Size,
        src: AddressingMode,
        dst: AddressingMode,
    },
    Movea {
        size: Size,  // Only Word or Long
        src: AddressingMode,
        reg: u8,
    },
    Moveq {
        data: i8,
        reg: u8,
    },
    Lea {
        src: AddressingMode,
        reg: u8,
    },
    Pea {
        src: AddressingMode,
    },
    
    // Arithmetic
    Add {
        size: Size,
        src: AddressingMode,
        dst: AddressingMode,
    },
    Adda {
        size: Size,  // Only Word or Long
        src: AddressingMode,
        reg: u8,
    },
    Addi {
        size: Size,
        imm: u32,
        dst: AddressingMode,
    },
    Addq {
        size: Size,
        data: u8,  // 1-8
        dst: AddressingMode,
    },
    Sub {
        size: Size,
        src: AddressingMode,
        dst: AddressingMode,
    },
    Suba {
        size: Size,
        src: AddressingMode,
        reg: u8,
    },
    Subi {
        size: Size,
        imm: u32,
        dst: AddressingMode,
    },
    Subq {
        size: Size,
        data: u8,
        dst: AddressingMode,
    },
    
    // Logical
    And {
        size: Size,
        src: AddressingMode,
        dst: AddressingMode,
    },
    Or {
        size: Size,
        src: AddressingMode,
        dst: AddressingMode,
    },
    Eor {
        size: Size,
        src: AddressingMode,
        dst: AddressingMode,
    },
    Not {
        size: Size,
        dst: AddressingMode,
    },
    
    // Branches
    Bra {
        displacement: i32,
    },
    Bsr {
        displacement: i32,
    },
    Bcc {
        condition: ConditionCode,
        displacement: i32,
    },
    
    // System
    Nop,
    Rts,
    Rte,
    Trap {
        vector: u8,
    },
    
    // Special
    Illegal {
        opcode: u16,
    },
}

/// Decode a 68k instruction from a 16-bit opcode
pub fn decode_instruction(opcode: u16) -> Instruction {
    // Extract common fields
    let op_high = (opcode >> 12) & 0xF;
    let reg_dst = ((opcode >> 9) & 0x7) as u8;
    let mode = ((opcode >> 6) & 0x7) as u8;
    let reg_src = (opcode & 0x7) as u8;
    
    match op_high {
        // 0x0: Bit manipulation, MOVEP, Immediate
        0x0 => {
            if opcode == 0x003C {
                // ORI to CCR
                return Instruction::Illegal { opcode };
            }
            if opcode == 0x023C {
                // ANDI to CCR
                return Instruction::Illegal { opcode };
            }
            if opcode == 0x0A3C {
                // EORI to CCR
                return Instruction::Illegal { opcode };
            }
            
            // For now, return illegal
            Instruction::Illegal { opcode }
        }
        
        // 0x1, 0x2, 0x3: MOVE
        0x1 | 0x2 | 0x3 => {
            let size = match op_high {
                0x1 => Size::Byte,
                0x2 => Size::Long,
                0x3 => Size::Word,
                _ => unreachable!(),
            };
            
            // TODO: Decode source and destination
            Instruction::Illegal { opcode }
        }
        
        // 0x4: Miscellaneous
        0x4 => {
            if opcode == 0x4E71 {
                return Instruction::Nop;
            }
            if opcode == 0x4E75 {
                return Instruction::Rts;
            }
            if opcode == 0x4E73 {
                return Instruction::Rte;
            }
            if (opcode & 0xFFF0) == 0x4E40 {
                return Instruction::Trap {
                    vector: (opcode & 0x0F) as u8,
                };
            }
            
            Instruction::Illegal { opcode }
        }
        
        // 0x5: ADDQ, SUBQ, Scc, DBcc
        0x5 => {
            let data = if reg_dst == 0 { 8 } else { reg_dst };
            let size_bits = (opcode >> 6) & 0x3;
            
            if let Some(size) = Size::from_bits(size_bits as u8) {
                let op_bit = (opcode >> 8) & 1;
                
                // TODO: Decode destination
                if op_bit == 0 {
                    return Instruction::Addq {
                        size,
                        data,
                        dst: AddressingMode::DataRegister(reg_src),
                    };
                } else {
                    return Instruction::Subq {
                        size,
                        data,
                        dst: AddressingMode::DataRegister(reg_src),
                    };
                }
            }
            
            Instruction::Illegal { opcode }
        }
        
        // 0x6: Bcc, BSR, BRA
        0x6 => {
            let condition = ConditionCode::from_bits(reg_dst);
            let displacement = (opcode & 0xFF) as i8 as i32;
            
            // displacement == 0 means next word contains 16-bit displacement
            // displacement == -1 means next long contains 32-bit displacement
            // (We'll handle these in the main decode loop with extension words)
            
            match condition {
                ConditionCode::False => Instruction::Bsr { displacement },
                ConditionCode::True => Instruction::Bra { displacement },
                _ => Instruction::Bcc { condition, displacement },
            }
        }
        
        // 0x7: MOVEQ
        0x7 => {
            // MOVEQ only if bit 8 is 0
            if (opcode & 0x0100) == 0 {
                let data = (opcode & 0xFF) as i8;
                return Instruction::Moveq {
                    data,
                    reg: reg_dst,
                };
            }
            Instruction::Illegal { opcode }
        }
        
        // 0xD: ADD, ADDA
        0xD => {
            // TODO: Full decoding
            Instruction::Illegal { opcode }
        }
        
        // 0x9: SUB, SUBA
        0x9 => {
            // TODO: Full decoding
            Instruction::Illegal { opcode }
        }
        
        // Everything else
        _ => Instruction::Illegal { opcode },
    }
}
