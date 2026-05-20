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

use super::forth::{ForthInterpreter, ForthWord};
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

impl OpenFirmwareForthExt for ForthInterpreter {
    fn register_of_words(&mut self) {
        // String operations
        self.register_primitive("$=", |i| i.string_equal());
        self.register_primitive("$find", |i| i.string_find());
        self.register_primitive("$call-method", |i| i.call_method());
        self.register_primitive("/\"", |i| i.parse_path_component());
        
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
    // Device tree operations
    // ============================================================================
    
    fn find_package(&mut self) -> Result<()> {
        // ( addr len -- phandle | 0 )
        // Find package by path, return phandle or 0
        let len = self.pop()? as usize;
        let addr = self.pop()? as usize;
        
        // Get package path
        let path = String::from_utf8_lossy(&self.data_space()[addr..addr+len]).to_string();
        tracing::debug!("Forth: find-package '{}'", path);
        
        // Calculate phandle from path
        let phandle = if path == "/" {
            1
        } else if path.starts_with('/') {
            // Return hash-based phandle
            path.bytes().fold(2i32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as i32))
        } else {
            0 // Not found
        };
        
        self.push(phandle);
        Ok(())
    }
    
    fn dev(&mut self) -> Result<()> {
        // dev ( addr len -- )
        // Open device tree node (short form of find-device)
        let len = self.pop()? as usize;
        let addr = self.pop()? as usize;
        
        // Get device path from data space
        let path = String::from_utf8_lossy(&self.data_space()[addr..addr+len]).to_string();
        tracing::debug!("Forth: dev '{}'", path);
        
        // Set current device path
        self.set_device_path(path);
        Ok(())
    }
    
    fn find_device(&mut self) -> Result<()> {
        // ( addr len -- )
        // Find and select device
        let len = self.pop()? as usize;
        let addr = self.pop()? as usize;
        
        // Get device path from data space
        let path = String::from_utf8_lossy(&self.data_space()[addr..addr+len]).to_string();
        tracing::debug!("Forth: find-device '{}'", path);
        
        // Set current device path
        self.set_device_path(path);
        Ok(())
    }
    
    fn get_package_property(&mut self) -> Result<()> {
        // ( phandle addr len -- addr' len' true | false )
        // Get property from package
        let name_len = self.pop()? as usize;
        let name_addr = self.pop()? as usize;
        let _phandle = self.pop()?;
        
        // Get property name
        let prop_name = String::from_utf8_lossy(&self.data_space()[name_addr..name_addr+name_len]).to_string();
        
        // Try to get property from current device
        let current_path = self.get_device_path().to_string();
        if let Some(value) = self.get_device_property(&current_path, &prop_name).map(|v| v.to_vec()) {
            // Property found - store in data space and return addr/len/true
            let addr = self.here_ptr();
            let len = value.len();
            
            if addr + len > self.data_space().len() {
                return Err(newton_utils::Error::Other("Data space exhausted".to_string()));
            }
            
            self.data_space_mut()[addr..addr+len].copy_from_slice(&value);
            self.set_here(addr + len);
            
            self.push(addr as i32);
            self.push(len as i32);
            self.push(-1); // true
        } else {
            // Property not found
            self.push(0); // false
        }
        Ok(())
    }
    
    fn active_package(&mut self) -> Result<()> {
        // ( -- phandle )
        // Return current active package
        // Use hash of current path as phandle
        let path = self.get_device_path();
        let phandle = if path == "/" {
            1
        } else {
            // Simple hash of path for phandle
            path.bytes().fold(2i32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as i32))
        };
        self.push(phandle);
        Ok(())
    }
    
    fn property(&mut self) -> Result<()> {
        // ( addr len name-addr name-len -- )
        // Create property in current device
        let name_len = self.pop()? as usize;
        let name_addr = self.pop()? as usize;
        let value_len = self.pop()? as usize;
        let value_addr = self.pop()? as usize;
        
        // Get property name and value
        let name = String::from_utf8_lossy(&self.data_space()[name_addr..name_addr+name_len]).to_string();
        let value = self.data_space()[value_addr..value_addr+value_len].to_vec();
        
        tracing::debug!("Forth: property '{}' = {} bytes", name, value_len);
        
        // Add property to current device
        let current_path = self.get_device_path().to_string();
        self.add_device_property(&current_path, &name, value);
        Ok(())
    }
    
    fn device_name(&mut self) -> Result<()> {
        // ( addr len -- )
        // Set name of new device (called after new-device)
        let len = self.pop()? as usize;
        let addr = self.pop()? as usize;
        
        // Get device name
        let name = String::from_utf8_lossy(&self.data_space()[addr..addr+len]).to_string();
        tracing::debug!("Forth: device-name '{}'", name);
        
        if self.is_creating_device() {
            // We're in new-device mode - create the child path and navigate to it
            self.set_new_device_name(name.clone());
            
            // Also store as "name" property
            let current_path = self.get_device_path().to_string();
            self.add_device_property(&current_path, "name", name.into_bytes());
            
            tracing::debug!("Forth: created device at '{}'", self.get_device_path());
        } else {
            // Just set the name property on current device
            let current_path = self.get_device_path().to_string();
            self.add_device_property(&current_path, "name", name.into_bytes());
        }
        Ok(())
    }
    
    fn device_end(&mut self) -> Result<()> {
        // ( -- )
        // End device node - return to parent
        let current_path = self.get_device_path().to_string();
        
        tracing::debug!("Forth: device-end from '{}'", current_path);
        
        // Navigate to parent
        if let Some(last_slash) = current_path.rfind('/') {
            let parent_path = if last_slash == 0 {
                "/".to_string()
            } else {
                current_path[..last_slash].to_string()
            };
            tracing::debug!("Forth: device-end, returning to '{}'", parent_path);
            self.set_device_path(parent_path);
        }
        Ok(())
    }
    
    fn new_device(&mut self) -> Result<()> {
        // ( -- )
        // Begin creating new device node
        // The name will be set with device-name
        tracing::debug!("Forth: new-device (parent: '{}')", self.get_device_path());
        self.begin_new_device();
        Ok(())
    }
    
    fn finish_device(&mut self) -> Result<()> {
        // ( -- )
        // Finish creating device - the device is now complete
        let current_path = self.get_device_path().to_string();
        tracing::debug!("Forth: finish-device at '{}'", current_path);
        
        // The device is complete - return to parent
        if let Some(last_slash) = current_path.rfind('/') {
            let parent_path = if last_slash == 0 {
                "/".to_string()
            } else {
                current_path[..last_slash].to_string()
            };
            self.set_device_path(parent_path);
        }
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
        // Return 0 if execution succeeds, non-zero exception code if it fails
        
        let xt = self.pop()? as usize;
        
        // Read length
        if xt + 4 > self.data_space().len() {
            self.push(-1); // Invalid XT exception
            return Ok(());
        }
        
        let len = i32::from_be_bytes([
            self.data_space()[xt],
            self.data_space()[xt+1],
            self.data_space()[xt+2],
            self.data_space()[xt+3],
        ]) as usize;
        
        if xt + 4 + len > self.data_space().len() {
            self.push(-2); // Invalid XT exception
            return Ok(());
        }
        
        // Read word name
        let name = String::from_utf8_lossy(&self.data_space()[xt+4..xt+4+len]).to_string();
        
        // Try to execute the word and catch any errors
        match self.execute_word(&name) {
            Ok(()) => {
                self.push(0); // Success - no exception
                Ok(())
            }
            Err(e) => {
                // Exception occurred - push non-zero exception code
                // Use -3 as a general exception code for now
                tracing::debug!("Forth: catch caught exception: {}", e);
                self.push(-3);
                Ok(())
            }
        }
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
        // begin - Mark loop start
        if self.is_compiling() {
            self.control_push(self.compile_len());
            self.compile_push("(begin)".to_string());
        }
        Ok(())
    }
    
    fn while_word(&mut self) -> Result<()> {
        // while ( flag -- ) - Exit loop if flag is false
        if self.is_compiling() {
            self.control_push(self.compile_len());
            self.compile_push("(while)".to_string());
            self.compile_push("0".to_string()); // Placeholder for jump offset
        } else {
            // Runtime: pop flag, skip if false
            let flag = self.pop()?;
            if flag == 0 {
                // Jump handled by word execution
            }
        }
        Ok(())
    }
    
    fn repeat_word(&mut self) -> Result<()> {
        // repeat - Jump back to BEGIN
        if self.is_compiling() {
            let while_pos = self.control_pop()
                .ok_or_else(|| newton_utils::Error::Other("REPEAT without WHILE".to_string()))?;
            let begin_pos = self.control_pop()
                .ok_or_else(|| newton_utils::Error::Other("REPEAT without BEGIN".to_string()))?;
            
            // Add unconditional jump back to BEGIN
            let back_offset = self.compile_len() - begin_pos + 1;
            self.compile_push("(repeat)".to_string());
            self.compile_push(format!("-{}", back_offset));
            
            // Back-patch WHILE to jump past REPEAT
            let forward_offset = self.compile_len() - while_pos - 2;
            self.compile_set(while_pos + 1, forward_offset.to_string());
        }
        Ok(())
    }
    
    fn question_do(&mut self) -> Result<()> {
        // ?do ( limit index -- ) - Start counted loop, skip if limit==index
        if self.is_compiling() {
            self.control_push(self.compile_len());
            self.compile_push("(?do)".to_string());
            self.compile_push("0".to_string()); // Placeholder
        } else {
            // Runtime
            let index = self.pop()?;
            let limit = self.pop()?;
            if index != limit {
                self.loop_push(index, limit);
            }
        }
        Ok(())
    }
    
    fn loop_word(&mut self) -> Result<()> {
        // loop - Increment index and loop if index < limit
        if self.is_compiling() {
            let do_pos = self.control_pop()
                .ok_or_else(|| newton_utils::Error::Other("LOOP without ?DO".to_string()))?;
            
            // Add loop instruction that jumps back to DO
            let back_offset = self.compile_len() - do_pos + 1;
            self.compile_push("(loop)".to_string());
            self.compile_push(format!("-{}", back_offset));
            
            // Back-patch ?DO to jump past LOOP if equal
            let forward_offset = self.compile_len() - do_pos - 2;
            self.compile_set(do_pos + 1, forward_offset.to_string());
        } else {
            // Runtime: increment and check
            if !self.loop_increment() {
                self.loop_pop();
            }
        }
        Ok(())
    }
    
    fn loop_index(&mut self) -> Result<()> {
        // i - get current loop index
        if let Some(idx) = self.get_loop_index() {
            self.push(idx);
        } else {
            self.push(0); // No loop active
        }
        Ok(())
    }
    
    // ============================================================================
    // Value operations
    // ============================================================================
    
    fn value_word(&mut self) -> Result<()> {
        // value ( n "name" -- ) - Create mutable value
        // Read next token as name
        let name = self.next_token()
            .ok_or_else(|| newton_utils::Error::Other("Expected value name".to_string()))?;
        let value = self.pop()?;
        
        // Values are stored like variables (in data space)
        let addr = self.here_ptr();
        if addr + 4 > self.data_space().len() {
            return Err(newton_utils::Error::Other("Data space exhausted".to_string()));
        }
        
        let bytes = value.to_be_bytes();
        self.data_space_mut()[addr..addr+4].copy_from_slice(&bytes);
        self.set_here(addr + 4);
        
        // Store as variable in dictionary
        self.create_variable(&name);
        Ok(())
    }
    
    fn to_word(&mut self) -> Result<()> {
        // ( n -- ) to name - Store to value
        // Read next token as name
        let name = self.next_token()
            .ok_or_else(|| newton_utils::Error::Other("Expected value name".to_string()))?;
        let value = self.pop()?;
        
        // Look up the variable address
        if let Some(ForthWord::Variable(addr)) = self.dictionary_get(&name).cloned() {
            let bytes = value.to_be_bytes();
            self.data_space_mut()[addr..addr+4].copy_from_slice(&bytes);
            Ok(())
        } else {
            Err(newton_utils::Error::Other(format!("Value not found: {}", name)))
        }
    }
    
    // ============================================================================
    // Execution tokens
    // ============================================================================
    
    fn bracket_tick(&mut self) -> Result<()> {
        // ['] name - Get execution token for word
        // Read next token as word name
        let name = self.next_token()
            .ok_or_else(|| newton_utils::Error::Other("Expected word name after [']".to_string()))?;
        
        // Check if word exists in dictionary
        if !self.dictionary_contains(&name) {
            return Err(newton_utils::Error::Other(format!("Word not found: {}", name)));
        }
        
        // Store word name in data space and push address
        let addr = self.here_ptr();
        let bytes = name.as_bytes();
        let len = bytes.len();
        
        if addr + len + 4 > self.data_space().len() {
            return Err(newton_utils::Error::Other("Data space exhausted".to_string()));
        }
        
        // Store length prefix (4 bytes)
        let len_bytes = (len as i32).to_be_bytes();
        self.data_space_mut()[addr..addr+4].copy_from_slice(&len_bytes);
        // Store name
        self.data_space_mut()[addr+4..addr+4+len].copy_from_slice(bytes);
        self.set_here(addr + 4 + len);
        
        // Push address as execution token
        self.push(addr as i32);
        Ok(())
    }
    
    fn execute(&mut self) -> Result<()> {
        // ( xt -- ) - Execute word by execution token
        let xt = self.pop()? as usize;
        
        // Read length
        if xt + 4 > self.data_space().len() {
            return Err(newton_utils::Error::Other("Invalid execution token".to_string()));
        }
        
        let len = i32::from_be_bytes([
            self.data_space()[xt],
            self.data_space()[xt+1],
            self.data_space()[xt+2],
            self.data_space()[xt+3],
        ]) as usize;
        
        if xt + 4 + len > self.data_space().len() {
            return Err(newton_utils::Error::Other("Invalid execution token".to_string()));
        }
        
        // Read word name
        let name = String::from_utf8_lossy(&self.data_space()[xt+4..xt+4+len]).to_string();
        
        // Execute the word
        self.execute_word(&name)?;
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
    
    fn parse_path_component(&mut self) -> Result<()> {
        // /" ( addr1 len1 -- addr2 len2 addr3 len3 )
        // Parse next path component from device path
        // addr1 len1 is the input string
        // addr2 len2 is the remaining string after the component
        // addr3 len3 is the component itself
        let len1 = self.pop()? as usize;
        let addr1 = self.pop()? as usize;
        
        if len1 == 0 {
            // Empty string - return empty component
            self.push(addr1 as i32);
            self.push(0);
            self.push(addr1 as i32);
            self.push(0);
            return Ok(());
        }
        
        let data = self.data_space();
        let string = &data[addr1..addr1 + len1];
        
        // Find next '/' or end of string
        let mut component_len = 0;
        for (i, &byte) in string.iter().enumerate() {
            if byte == b'/' {
                component_len = i;
                break;
            }
            component_len = i + 1;
        }
        
        // Determine if we found a '/'
        let has_slash = component_len < len1 && string[component_len] == b'/';
        
        // Remaining string starts after component and optional '/'
        let remaining_start = if has_slash {
            addr1 + component_len + 1
        } else {
            addr1 + component_len
        };
        let remaining_len = if has_slash {
            len1 - component_len - 1
        } else {
            len1 - component_len
        };
        
        // Push remaining string
        self.push(remaining_start as i32);
        self.push(remaining_len as i32);
        
        // Push component
        self.push(addr1 as i32);
        self.push(component_len as i32);
        
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
