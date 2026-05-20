// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! OpenFirmware-specific Forth words
//!
//! This module extends the base Forth interpreter with OpenFirmware-specific
//! words needed for boot scripts and device tree manipulation.

use super::forth::ForthInterpreter;
use super::device_tree::DeviceTree;
use super::client_interface::ClientInterface;
use newton_utils::Result;

/// Extension trait for OpenFirmware-specific Forth words
pub trait OpenFirmwareForthExt {
    /// Register OpenFirmware-specific words in the interpreter
    fn register_of_words(&mut self);
    
    /// Set reference to device tree and client interface
    fn set_of_context(&mut self, device_tree: *mut DeviceTree, client: *mut ClientInterface);
}

/// OpenFirmware context stored in the interpreter
pub struct OFContext {
    pub device_tree: *mut DeviceTree,
    pub client_interface: *mut ClientInterface,
    pub current_package: u32,  // phandle of current/active package
    pub program_entry: Option<u32>,  // Entry point set by init-program
    pub load_base: Option<u32>,  // Load base address
}

impl ForthInterpreter {
    /// Get OpenFirmware context (stored at a special location)
    fn of_context(&self) -> Option<&OFContext> {
        // For now, return None - we'll add proper context storage later
        None
    }
    
    /// Get OpenFirmware context mutably
    fn of_context_mut(&mut self) -> Option<&mut OFContext> {
        // For now, return None - we'll add proper context storage later
        None
    }
}

impl OpenFirmwareForthExt for ForthInterpreter {
    fn register_of_words(&mut self) {
        // String operations
        self.register_primitive("$=", |i| i.string_equal());
        self.register_primitive("$find", |i| i.string_find());
        self.register_primitive("$call-method", |i| i.call_method());
        
        // String encoding/decoding
        self.register_primitive("decode-string", |i| i.decode_string());
        self.register_primitive("decode-int", |i| i.decode_int());
        self.register_primitive("encode-int", |i| i.encode_int());
        self.register_primitive("encode+", |i| i.encode_plus());
        self.register_primitive("encode-string", |i| i.encode_string());
        
        // String parsing
        self.register_primitive("bounds", |i| i.bounds());
        self.register_primitive("[char]", |i| i.bracket_char());
        self.register_primitive("between", |i| i.between());
        
        // Memory block operations
        self.register_primitive("move", |i| i.move_memory());
        self.register_primitive("fill", |i| i.fill_memory());
        
        // Device tree operations
        self.register_primitive("dev", |i| i.dev());
        self.register_primitive("find-package", |i| i.find_package());
        self.register_primitive("find-device", |i| i.find_device());
        self.register_primitive("get-package-property", |i| i.get_package_property());
        self.register_primitive("active-package", |i| i.active_package());
        self.register_primitive("property", |i| i.property());
        self.register_primitive("device-name", |i| i.device_name());
        self.register_primitive("device-end", |i| i.device_end());
        self.register_primitive("new-device", |i| i.new_device());
        self.register_primitive("finish-device", |i| i.finish_device());
        
        // Error handling
        self.register_primitive("abort", |i| i.abort());
        self.register_primitive("catch", |i| i.catch());
        
        // Control flow extensions
        self.register_primitive("?dup", |i| i.question_dup());
        self.register_primitive("begin", |i| i.begin_word());
        self.register_primitive("while", |i| i.while_word());
        self.register_primitive("repeat", |i| i.repeat_word());
        self.register_primitive("?do", |i| i.question_do());
        self.register_primitive("loop", |i| i.loop_word());
        self.register_primitive("i", |i| i.loop_index());
        
        // Value operations (mutable constants)
        self.register_primitive("value", |i| i.value_word());
        self.register_primitive("to", |i| i.to_word());
        
        // Execution tokens
        self.register_primitive("[']", |i| i.bracket_tick());
        self.register_primitive("execute", |i| i.execute());
        
        // Additional stack ops
        self.register_primitive("3drop", |i| i.three_drop());
        
        // String evaluation
        self.register_primitive("eval", |i| i.eval_string());
        
        // Checksums
        self.register_primitive("adler32", |i| i.adler32());
        
        // Program execution (stubs for now)
        self.register_primitive("init-program", |i| i.init_program());
        self.register_primitive("go", |i| i.go());
        self.register_primitive(".registers", |i| i.dot_registers());
    }
    
    fn set_of_context(&mut self, _device_tree: *mut DeviceTree, _client: *mut ClientInterface) {
        // TODO: Store context pointers
    }
}

impl ForthInterpreter {
    // ============================================================================
    // String operations
    // ============================================================================
    
    fn string_equal(&mut self) -> Result<()> {
        // ( addr1 len1 addr2 len2 -- flag )
        // Compare two strings
        let len2 = self.pop()? as usize;
        let addr2 = self.pop()? as usize;
        let len1 = self.pop()? as usize;
        let addr1 = self.pop()? as usize;
        
        if len1 != len2 {
            self.push(0);
            return Ok(());
        }
        
        let equal = self.data_space()[addr1..addr1+len1] == self.data_space()[addr2..addr2+len2];
        self.push(if equal { -1 } else { 0 });
        Ok(())
    }
    
    fn string_find(&mut self) -> Result<()> {
        // ( addr len -- xt flag | 0 0 )
        // Find word in dictionary, return execution token and true, or 0 0 if not found
        // For now, simplified implementation
        self.push(0);
        self.push(0);
        Ok(())
    }
    
    fn decode_string(&mut self) -> Result<()> {
        // ( addr1 len1 -- addr1+n len1-n addr2 len2 )
        // Decode a string from a properties buffer
        // First cell is length, followed by string bytes
        let len1 = self.pop()? as usize;
        let addr1 = self.pop()? as usize;
        
        if len1 < 4 {
            return Err(newton_utils::Error::Other("Buffer too small for decode-string".to_string()));
        }
        
        // Read length (big-endian 32-bit)
        let data = self.data_space();
        let str_len = i32::from_be_bytes([
            data[addr1],
            data[addr1 + 1],
            data[addr1 + 2],
            data[addr1 + 3],
        ]) as usize;
        
        // Align to 4 bytes
        let aligned_len = (str_len + 3) & !3;
        let total = 4 + aligned_len;
        
        if total > len1 {
            return Err(newton_utils::Error::Other("Buffer too small for string".to_string()));
        }
        
        // Push remaining buffer
        self.push((addr1 + total) as i32);
        self.push((len1 - total) as i32);
        
        // Push decoded string
        self.push((addr1 + 4) as i32);
        self.push(str_len as i32);
        
        Ok(())
    }
    
    fn decode_int(&mut self) -> Result<()> {
        // ( addr1 len1 -- addr1+4 len1-4 value )
        // Decode a 32-bit integer from buffer
        let len1 = self.pop()? as usize;
        let addr1 = self.pop()? as usize;
        
        if len1 < 4 {
            return Err(newton_utils::Error::Other("Buffer too small for decode-int".to_string()));
        }
        
        let data = self.data_space();
        let value = i32::from_be_bytes([
            data[addr1],
            data[addr1 + 1],
            data[addr1 + 2],
            data[addr1 + 3],
        ]);
        
        self.push((addr1 + 4) as i32);
        self.push((len1 - 4) as i32);
        self.push(value);
        
        Ok(())
    }
    
    fn encode_int(&mut self) -> Result<()> {
        // ( value -- addr len )
        // Encode integer as big-endian bytes in data space
        let value = self.pop()?;
        let addr = self.here_ptr();
        
        let bytes = value.to_be_bytes();
        if addr + 4 > self.data_space().len() {
            return Err(newton_utils::Error::Other("Data space exhausted".to_string()));
        }
        
        let here = self.here_ptr();
        self.data_space_mut()[here..here+4].copy_from_slice(&bytes);
        self.set_here(here + 4);
        
        self.push(addr as i32);
        self.push(4);
        
        Ok(())
    }
    
    fn encode_plus(&mut self) -> Result<()> {
        // ( addr1 len1 addr2 len2 -- addr3 len3 )
        // Concatenate two encoded properties
        let len2 = self.pop()? as usize;
        let addr2 = self.pop()? as usize;
        let len1 = self.pop()? as usize;
        let addr1 = self.pop()? as usize;
        
        let total_len = len1 + len2;
        let dest_addr = self.here_ptr();
        
        if self.here_ptr() + total_len > self.data_space().len() {
            return Err(newton_utils::Error::Other("Data space exhausted".to_string()));
        }
        
        // Copy both buffers
        let data = self.data_space();
        let buf1 = data[addr1..addr1+len1].to_vec();
        let buf2 = data[addr2..addr2+len2].to_vec();
        
        let here1 = self.here_ptr();
        let dest_data = self.data_space_mut();
        dest_data[here1..here1+len1].copy_from_slice(&buf1);
        self.set_here(here1 + len1);
        
        let here2 = self.here_ptr();
        let dest_data = self.data_space_mut();
        dest_data[here2..here2+len2].copy_from_slice(&buf2);
        self.set_here(here2 + len2);
        
        self.push(dest_addr as i32);
        self.push(total_len as i32);
        
        Ok(())
    }
    
    fn encode_string(&mut self) -> Result<()> {
        // ( addr len -- addr' len' )
        // Encode string with length prefix
        let len = self.pop()? as usize;
        let addr = self.pop()? as usize;
        
        let dest_addr = self.here_ptr();
        let aligned_len = (len + 3) & !3;
        let total_len = 4 + aligned_len;
        
        if dest_addr + total_len > self.data_space().len() {
            return Err(newton_utils::Error::Other("Data space exhausted".to_string()));
        }
        
        // Write length
        let len_bytes = (len as i32).to_be_bytes();
        let here1 = self.here_ptr();
        self.data_space_mut()[here1..here1+4].copy_from_slice(&len_bytes);
        self.set_here(here1 + 4);
        
        // Write string
        let data = self.data_space();
        let string_data = data[addr..addr+len].to_vec();
        let here2 = self.here_ptr();
        self.data_space_mut()[here2..here2+len].copy_from_slice(&string_data);
        self.set_here(here2 + aligned_len);
        
        self.push(dest_addr as i32);
        self.push(total_len as i32);
        
        Ok(())
    }
    
    fn bounds(&mut self) -> Result<()> {
        // ( addr len -- addr+len addr )
        // Convert address and length to end and start for DO loop
        let len = self.pop()?;
        let addr = self.pop()?;
        self.push(addr + len);
        self.push(addr);
        Ok(())
    }
    
    fn bracket_char(&mut self) -> Result<()> {
        // [char] c -- compiles character code
        // For now, simplified
        Ok(())
    }
    
    fn between(&mut self) -> Result<()> {
        // ( n low high -- flag )
        // Check if n is between low and high inclusive
        let high = self.pop()?;
        let low = self.pop()?;
        let n = self.pop()?;
        self.push(if n >= low && n <= high { -1 } else { 0 });
        Ok(())
    }
    
    // ============================================================================
    // Memory block operations
    // ============================================================================
    
    fn move_memory(&mut self) -> Result<()> {
        // ( src dest len -- )
        // Copy len bytes from src to dest
        let len = self.pop()? as usize;
        let dest = self.pop()? as usize;
        let src = self.pop()? as usize;
        
        if src + len > self.data_space().len() || dest + len > self.data_space().len() {
            return Err(newton_utils::Error::Other("Memory access out of bounds".to_string()));
        }
        
        // Copy data through a temporary buffer to handle overlaps
        let data = self.data_space();
        let src_data = data[src..src+len].to_vec();
        self.data_space_mut()[dest..dest+len].copy_from_slice(&src_data);
        
        Ok(())
    }
    
    fn fill_memory(&mut self) -> Result<()> {
        // ( addr len byte -- )
        // Fill len bytes at addr with byte
        let byte = self.pop()? as u8;
        let len = self.pop()? as usize;
        let addr = self.pop()? as usize;
        
        if addr + len > self.data_space().len() {
            return Err(newton_utils::Error::Other("Memory access out of bounds".to_string()));
        }
        
        self.data_space_mut()[addr..addr+len].fill(byte);
        Ok(())
    }
    
    // ============================================================================
    // Device tree operations (stubs for now)
    // ============================================================================
    
    fn find_package(&mut self) -> Result<()> {
        // ( addr len -- phandle | 0 )
        // Find package by path, return phandle or 0
        // For now, simplified stub
        self.pop()?; // len
        self.pop()?; // addr
        self.push(1); // Return root phandle for now
        Ok(())
    }
    
    fn dev(&mut self) -> Result<()> {
        // dev ( addr len -- )
        // Open device tree node (short form of find-device)
        let len = self.pop()? as usize;
        let addr = self.pop()? as usize;
        
        // Get device path from data space
        let _path = String::from_utf8_lossy(&self.data_space()[addr..addr+len]);
        // tracing::debug!("Forth: dev '{}'", path);
        
        // TODO: Actually set current device
        Ok(())
    }
    
    fn find_device(&mut self) -> Result<()> {
        // ( addr len -- )
        // Find and select device
        self.pop()?; // len
        self.pop()?; // addr
        Ok(())
    }
    
    fn get_package_property(&mut self) -> Result<()> {
        // ( phandle addr len -- addr' len' true | false )
        // Get property from package
        // For now, return false (property not found)
        self.pop()?; // len
        self.pop()?; // addr
        self.pop()?; // phandle
        self.push(0); // false
        Ok(())
    }
    
    fn active_package(&mut self) -> Result<()> {
        // ( -- phandle )
        // Return current active package
        self.push(1); // Return root for now
        Ok(())
    }
    
    fn property(&mut self) -> Result<()> {
        // ( addr len name-addr name-len -- )
        // Create property in current device
        self.pop()?; // name len
        self.pop()?; // name addr
        self.pop()?; // value len
        self.pop()?; // value addr
        Ok(())
    }
    
    fn device_name(&mut self) -> Result<()> {
        // ( addr len -- )
        // Set name of new device
        self.pop()?; // len
        self.pop()?; // addr
        Ok(())
    }
    
    fn device_end(&mut self) -> Result<()> {
        // ( -- )
        // End device node
        Ok(())
    }
    
    fn new_device(&mut self) -> Result<()> {
        // ( -- )
        // Create new device node
        Ok(())
    }
    
    fn finish_device(&mut self) -> Result<()> {
        // ( -- )
        // Finish creating device
        Ok(())
    }
    
    // ============================================================================
    // Error handling
    // ============================================================================
    
    fn abort(&mut self) -> Result<()> {
        // ( -- )
        // Abort with error message
        Err(newton_utils::Error::Other("ABORT".to_string()))
    }
    
    fn catch(&mut self) -> Result<()> {
        // ( xt -- exception# | 0 )
        // Execute xt and catch exceptions
        // For now, simplified stub
        self.pop()?; // xt
        self.push(0); // No exception
        Ok(())
    }
    
    // ============================================================================
    // Control flow extensions
    // ============================================================================
    
    fn question_dup(&mut self) -> Result<()> {
        // ( n -- n n | 0 )
        // Duplicate if non-zero
        let val = self.peek()?;
        if val != 0 {
            self.push(val);
        }
        Ok(())
    }
    
    fn begin_word(&mut self) -> Result<()> {
        // Begin loop - stub
        Ok(())
    }
    
    fn while_word(&mut self) -> Result<()> {
        // While condition - stub
        Ok(())
    }
    
    fn repeat_word(&mut self) -> Result<()> {
        // Repeat loop - stub
        Ok(())
    }
    
    fn question_do(&mut self) -> Result<()> {
        // ?do loop - stub
        Ok(())
    }
    
    fn loop_word(&mut self) -> Result<()> {
        // Loop - stub
        Ok(())
    }
    
    fn loop_index(&mut self) -> Result<()> {
        // i - get loop index
        self.push(0); // Stub
        Ok(())
    }
    
    // ============================================================================
    // Value operations
    // ============================================================================
    
    fn value_word(&mut self) -> Result<()> {
        // value name - create mutable value
        // Stub for now
        Ok(())
    }
    
    fn to_word(&mut self) -> Result<()> {
        // ( n -- ) to name - store to value
        // Stub for now
        self.pop()?;
        Ok(())
    }
    
    // ============================================================================
    // Execution tokens
    // ============================================================================
    
    fn bracket_tick(&mut self) -> Result<()> {
        // ['] name - get execution token at compile time
        // Stub for now
        self.push(0);
        Ok(())
    }
    
    fn execute(&mut self) -> Result<()> {
        // ( xt -- )
        // Execute word by execution token
        self.pop()?;
        Ok(())
    }
    
    // ============================================================================
    // Additional stack operations
    // ============================================================================
    
    fn three_drop(&mut self) -> Result<()> {
        // ( a b c -- )
        self.pop()?;
        self.pop()?;
        self.pop()?;
        Ok(())
    }
    
    // ============================================================================
    // Program execution
    // ============================================================================
    
    fn init_program(&mut self) -> Result<()> {
        // init-program ( -- )
        // Initialize program for execution
        // The boot script copies the ELF to load-base before calling this
        
        tracing::info!("Forth: init-program called");
        
        // The ELF should be at load-base
        // For now, we'll just mark that init-program was called
        // The actual ELF parsing should happen in the emulator
        // when it sees that init-program was called
        
        // Try to get load-base from dictionary
        let load_base_value = if let Some(load_base_word) = self.dictionary().get("load-base") {
            if let super::forth::ForthWord::Constant(addr) = load_base_word {
                Some(*addr as u32)
            } else {
                None
            }
        } else {
            None
        };
        
        if let Some(addr) = load_base_value {
            self.load_base = Some(addr);
            tracing::info!("  load-base set to 0x{:08X}", addr);
        }
        
        // Mark that we should initialize the program
        tracing::info!("  Program ready for initialization");
        tracing::info!("  (Actual ELF parsing happens in emulator)");
        
        Ok(())
    }
    
    fn go(&mut self) -> Result<()> {
        // go ( -- )
        // Start program execution
        // Transfers control to the program entry point set by init-program
        
        tracing::info!("Forth: go called");
        
        if let Some(load_base) = self.load_base {
            tracing::info!("  Would transfer control to program at 0x{:08X}", load_base);
            tracing::info!("  (Actual control transfer happens in emulator)");
            
            // Set a flag that the emulator can check
            self.program_entry = Some(load_base);
        } else {
            tracing::warn!("  No load-base set - init-program not called?");
        }
        
        Ok(())
    }
    
    fn call_method(&mut self) -> Result<()> {
        // $call-method ( ... method-str method-len ihandle -- ... )
        // Call a method on an OpenFirmware instance
        // For now, stub - return success
        let _ihandle = self.pop()?;
        let _method_len = self.pop()? as usize;
        let _method_addr = self.pop()? as usize;
        
        // Get method name from data space
        // let method = String::from_utf8_lossy(&self.data_space()[method_addr..method_addr+method_len]);
        // tracing::debug!("Forth: $call-method '{}' on ihandle 0x{:x}", method, ihandle);
        
        // TODO: Actually call the method through client interface
        // For now, push success (0)
        self.push(0);
        Ok(())
    }
    
    fn eval_string(&mut self) -> Result<()> {
        // eval ( addr len -- ??? )
        // Evaluate a string as Forth code
        let len = self.pop()? as usize;
        let addr = self.pop()? as usize;
        
        // Get string from data space
        let code = String::from_utf8_lossy(&self.data_space()[addr..addr+len]).to_string();
        
        // Evaluate it
        self.eval(&code)?;
        Ok(())
    }
    
    fn adler32(&mut self) -> Result<()> {
        // adler32 ( addr len init -- checksum )
        // Calculate Adler-32 checksum
        let init = self.pop()? as u32;
        let len = self.pop()? as usize;
        let addr = self.pop()? as usize;
        
        // Simple Adler-32 implementation
        let data = &self.data_space()[addr..addr+len];
        let mut a = (init & 0xFFFF) as u32;
        let mut b = ((init >> 16) & 0xFFFF) as u32;
        
        const MOD_ADLER: u32 = 65521;
        
        for &byte in data {
            a = (a + byte as u32) % MOD_ADLER;
            b = (b + a) % MOD_ADLER;
        }
        
        let checksum = ((b << 16) | a) as i32;
        self.push(checksum);
        Ok(())
    }
    
    fn dot_registers(&mut self) -> Result<()> {
        // Print register state (for debugging)
        tracing::info!("Forth: .registers called");
        Ok(())
    }
}
