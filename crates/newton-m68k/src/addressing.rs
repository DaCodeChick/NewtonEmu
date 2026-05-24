// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! 68k addressing mode calculation and operand access

use crate::{Registers, AddressingMode, Size, MemoryInterface};
use newton_utils::{Result, Error};

/// Calculate the effective address for an addressing mode
pub fn calculate_ea(
    mode: &AddressingMode,
    registers: &mut Registers,
    memory: &dyn MemoryInterface,
) -> Result<u32> {
    match mode {
        AddressingMode::DataRegister(_) => {
            Err(Error::Cpu("Cannot calculate EA for data register".into()))
        }
        
        AddressingMode::AddressRegister(_) => {
            Err(Error::Cpu("Cannot calculate EA for address register".into()))
        }
        
        AddressingMode::AddressIndirect(reg) => {
            Ok(registers.a[*reg as usize])
        }
        
        AddressingMode::AddressPostIncrement(reg) => {
            let addr = registers.a[*reg as usize];
            Ok(addr)
        }
        
        AddressingMode::AddressPreDecrement(reg) => {
            // Predecrement happens during access
            Ok(registers.a[*reg as usize])
        }
        
        AddressingMode::AddressDisplacement(reg, disp) => {
            let base = registers.a[*reg as usize];
            Ok(base.wrapping_add(*disp as i32 as u32))
        }
        
        AddressingMode::AddressIndex(reg, index_reg, index_size, disp) => {
            let base = registers.a[*reg as usize];
            let index = if (*index_reg & 0x08) != 0 {
                // Address register
                registers.a[(*index_reg & 0x07) as usize]
            } else {
                // Data register
                registers.d[(*index_reg & 0x07) as usize]
            };
            
            let index_value = match index_size {
                Size::Word => (index as i16) as i32 as u32,
                Size::Long => index,
                _ => return Err(Error::Cpu("Invalid index size".into())),
            };
            
            Ok(base.wrapping_add(index_value).wrapping_add(*disp as i32 as u32))
        }
        
        AddressingMode::AbsoluteShort(addr) => {
            Ok((*addr as i32) as u32)
        }
        
        AddressingMode::AbsoluteLong(addr) => {
            Ok(*addr)
        }
        
        AddressingMode::PcDisplacement(disp) => {
            let pc = registers.pc;
            Ok(pc.wrapping_add(*disp as i32 as u32))
        }
        
        AddressingMode::PcIndex(index_reg, index_size, disp) => {
            let pc = registers.pc;
            let index = if (*index_reg & 0x08) != 0 {
                registers.a[(*index_reg & 0x07) as usize]
            } else {
                registers.d[(*index_reg & 0x07) as usize]
            };
            
            let index_value = match index_size {
                Size::Word => (index as i16) as i32 as u32,
                Size::Long => index,
                _ => return Err(Error::Cpu("Invalid index size".into())),
            };
            
            Ok(pc.wrapping_add(index_value).wrapping_add(*disp as i32 as u32))
        }
        
        AddressingMode::Immediate(_) => {
            Err(Error::Cpu("Cannot calculate EA for immediate".into()))
        }
    }
}

/// Read an operand value based on addressing mode
pub fn read_operand(
    mode: &AddressingMode,
    size: Size,
    registers: &mut Registers,
    memory: &dyn MemoryInterface,
) -> Result<u32> {
    match mode {
        AddressingMode::DataRegister(reg) => {
            let value = registers.d[*reg as usize];
            Ok(match size {
                Size::Byte => value & 0xFF,
                Size::Word => value & 0xFFFF,
                Size::Long => value,
            })
        }
        
        AddressingMode::AddressRegister(reg) => {
            Ok(registers.a[*reg as usize])
        }
        
        AddressingMode::AddressPostIncrement(reg) => {
            let addr = registers.a[*reg as usize];
            let value = read_memory(memory, addr, size)?;
            
            // Post-increment
            let increment = if *reg == 7 && size == Size::Byte {
                // Stack pointer always increments by 2
                2
            } else {
                size.bytes()
            };
            registers.a[*reg as usize] = addr.wrapping_add(increment);
            
            Ok(value)
        }
        
        AddressingMode::AddressPreDecrement(reg) => {
            // Pre-decrement
            let decrement = if *reg == 7 && size == Size::Byte {
                2
            } else {
                size.bytes()
            };
            let addr = registers.a[*reg as usize].wrapping_sub(decrement);
            registers.a[*reg as usize] = addr;
            
            read_memory(memory, addr, size)
        }
        
        AddressingMode::Immediate(value) => {
            Ok(match size {
                Size::Byte => *value & 0xFF,
                Size::Word => *value & 0xFFFF,
                Size::Long => *value,
            })
        }
        
        _ => {
            let ea = calculate_ea(mode, registers, memory)?;
            read_memory(memory, ea, size)
        }
    }
}

/// Write an operand value based on addressing mode
pub fn write_operand(
    mode: &AddressingMode,
    size: Size,
    value: u32,
    registers: &mut Registers,
    memory: &dyn MemoryInterface,
) -> Result<()> {
    match mode {
        AddressingMode::DataRegister(reg) => {
            match size {
                Size::Byte => {
                    registers.d[*reg as usize] = (registers.d[*reg as usize] & 0xFFFFFF00) | (value & 0xFF);
                }
                Size::Word => {
                    registers.d[*reg as usize] = (registers.d[*reg as usize] & 0xFFFF0000) | (value & 0xFFFF);
                }
                Size::Long => {
                    registers.d[*reg as usize] = value;
                }
            }
            Ok(())
        }
        
        AddressingMode::AddressRegister(reg) => {
            registers.a[*reg as usize] = value;
            Ok(())
        }
        
        AddressingMode::AddressPostIncrement(reg) => {
            let addr = registers.a[*reg as usize];
            write_memory(memory, addr, size, value)?;
            
            let increment = if *reg == 7 && size == Size::Byte {
                2
            } else {
                size.bytes()
            };
            registers.a[*reg as usize] = addr.wrapping_add(increment);
            
            Ok(())
        }
        
        AddressingMode::AddressPreDecrement(reg) => {
            let decrement = if *reg == 7 && size == Size::Byte {
                2
            } else {
                size.bytes()
            };
            let addr = registers.a[*reg as usize].wrapping_sub(decrement);
            registers.a[*reg as usize] = addr;
            
            write_memory(memory, addr, size, value)
        }
        
        _ => {
            let ea = calculate_ea(mode, registers, memory)?;
            write_memory(memory, ea, size, value)
        }
    }
}

/// Read from memory with the specified size
fn read_memory(memory: &dyn MemoryInterface, addr: u32, size: Size) -> Result<u32> {
    match size {
        Size::Byte => Ok(memory.read_u8(addr)? as u32),
        Size::Word => Ok(memory.read_u16(addr)? as u32),
        Size::Long => Ok(memory.read_u32(addr)?),
    }
}

/// Write to memory with the specified size
fn write_memory(memory: &dyn MemoryInterface, addr: u32, size: Size, value: u32) -> Result<()> {
    match size {
        Size::Byte => memory.write_u8(addr, value as u8),
        Size::Word => memory.write_u16(addr, value as u16),
        Size::Long => memory.write_u32(addr, value),
    }
}

/// Update condition codes based on result
pub fn update_condition_codes(
    registers: &mut Registers,
    result: u32,
    size: Size,
    carry: bool,
    overflow: bool,
) {
    // Get the sign bit for the size
    let sign_bit = match size {
        Size::Byte => result & 0x80 != 0,
        Size::Word => result & 0x8000 != 0,
        Size::Long => result & 0x8000_0000 != 0,
    };
    
    // Get the masked result for zero check
    let masked = match size {
        Size::Byte => result & 0xFF,
        Size::Word => result & 0xFFFF,
        Size::Long => result,
    };
    
    registers.sr.set_negative(sign_bit);
    registers.sr.set_zero(masked == 0);
    registers.sr.set_carry(carry);
    registers.sr.set_overflow(overflow);
}
