// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! OpenFirmware client interface
//!
//! The client interface is called by ROM/OS code via a callback mechanism.
//! Arguments are passed in a structure in memory.

use super::DeviceTree;
use newton_utils::Result;
use std::collections::HashMap;

/// OpenFirmware client interface
///
/// Handles calls from ROM/OS to query the device tree and perform operations.
pub struct ClientInterface {
    /// Handle counter for device nodes
    next_handle: u32,
    
    /// Map from phandle to device path
    handle_to_path: HashMap<u32, String>,
    
    /// Map from device path to phandle
    path_to_handle: HashMap<String, u32>,
}

impl ClientInterface {
    /// Create a new client interface
    pub fn new() -> Self {
        Self {
            next_handle: 1,
            handle_to_path: HashMap::new(),
            path_to_handle: HashMap::new(),
        }
    }
    
    /// Call a client interface service
    ///
    /// The arguments structure in memory contains:
    /// - service name (string pointer)
    /// - n_args (number of input arguments)
    /// - n_returns (number of return values)
    /// - arguments (n_args values)
    /// - returns (n_returns values, filled by this function)
    pub fn call_service(
        &mut self,
        device_tree: &mut DeviceTree,
        service: &str,
        args: &[u32],
        string_args: &[String],
    ) -> Result<Vec<u32>> {
        tracing::debug!("OpenFirmware service call: {} with {} args", service, args.len());
        
        match service {
            // Device tree traversal
            "peer" => self.peer(device_tree, args),
            "child" => self.child(device_tree, args),
            "parent" => self.parent(device_tree, args),
            
            // Device node properties
            "getprop" => self.getprop(device_tree, args, string_args),
            "getproplen" => self.getproplen(device_tree, args, string_args),
            "nextprop" => self.nextprop(device_tree, args),
            "setprop" => self.setprop(device_tree, args),
            
            // Device node lookup
            "finddevice" => self.finddevice(device_tree, args, string_args),
            "package-to-path" => self.package_to_path(device_tree, args),
            
            // Memory allocation
            "claim" => self.claim(args),
            "release" => self.release(args),
            
            // I/O operations
            "open" => self.open(args),
            "close" => self.close(args),
            "read" => self.read(args),
            "write" => self.write(args),
            "seek" => self.seek(args),
            
            // Miscellaneous
            "exit" => self.exit(),
            "test" => self.test(args),
            
            _ => {
                tracing::warn!("Unimplemented OpenFirmware service: {}", service);
                Ok(vec![u32::MAX]) // Return failure
            }
        }
    }
    
    fn peer(&mut self, _device_tree: &DeviceTree, _args: &[u32]) -> Result<Vec<u32>> {
        // TODO: Implement peer navigation
        Ok(vec![0])
    }
    
    fn child(&mut self, _device_tree: &DeviceTree, _args: &[u32]) -> Result<Vec<u32>> {
        // TODO: Implement child navigation
        Ok(vec![0])
    }
    
    fn parent(&mut self, _device_tree: &DeviceTree, _args: &[u32]) -> Result<Vec<u32>> {
        // TODO: Implement parent navigation
        Ok(vec![0])
    }
    
    fn getprop(&self, device_tree: &DeviceTree, args: &[u32], string_args: &[String]) -> Result<Vec<u32>> {
        // args: [phandle, property_name_ptr, buf_ptr, buf_len]
        // returns: [actual_len]
        if args.len() < 4 || string_args.len() < 2 {
            return Ok(vec![u32::MAX]); // -1 = error
        }
        
        let phandle = args[0];
        let property_name = &string_args[1];
        let _buf_ptr = args[2];
        let _buf_len = args[3];
        
        tracing::debug!("getprop: phandle=0x{:08X}, property={}", phandle, property_name);
        
        // Look up the device path from the handle
        if let Some(path) = self.handle_to_path.get(&phandle) {
            // Get the property value
            if let Some(value) = device_tree.get_property(path, property_name) {
                // TODO: Write value to buf_ptr in memory
                // For now, just return the length
                let len = value.len() as u32;
                tracing::debug!("  -> Found property, len={}", len);
                return Ok(vec![len]);
            }
        }
        
        tracing::warn!("  -> Property not found");
        Ok(vec![u32::MAX]) // -1 = not found
    }
    
    fn getproplen(&self, device_tree: &DeviceTree, args: &[u32], string_args: &[String]) -> Result<Vec<u32>> {
        // args: [phandle, property_name_ptr]
        // returns: [len]
        if args.len() < 2 || string_args.len() < 2 {
            return Ok(vec![u32::MAX]);
        }
        
        let phandle = args[0];
        let property_name = &string_args[1];
        
        tracing::debug!("getproplen: phandle=0x{:08X}, property={}", phandle, property_name);
        
        // Look up the device path from the handle
        if let Some(path) = self.handle_to_path.get(&phandle) {
            // Get the property value
            if let Some(value) = device_tree.get_property(path, property_name) {
                let len = value.len() as u32;
                tracing::debug!("  -> len={}", len);
                return Ok(vec![len]);
            }
        }
        
        tracing::warn!("  -> Property not found");
        Ok(vec![u32::MAX]) // -1 = not found
    }
    
    fn nextprop(&self, _device_tree: &DeviceTree, _args: &[u32]) -> Result<Vec<u32>> {
        Ok(vec![u32::MAX])
    }
    
    fn setprop(&mut self, _device_tree: &mut DeviceTree, _args: &[u32]) -> Result<Vec<u32>> {
        Ok(vec![0])
    }
    
    fn finddevice(&mut self, device_tree: &DeviceTree, _args: &[u32], string_args: &[String]) -> Result<Vec<u32>> {
        // args: [device_path_ptr]
        // returns: [phandle]
        if string_args.is_empty() {
            return Ok(vec![u32::MAX]);
        }
        
        let device_path = &string_args[0];
        tracing::debug!("finddevice: path={}", device_path);
        
        // Check if we already have a handle for this path
        if let Some(&handle) = self.path_to_handle.get(device_path) {
            tracing::debug!("  -> Found existing handle 0x{:08X}", handle);
            return Ok(vec![handle]);
        }
        
        // Look up the device in the tree
        if device_tree.find_node(device_path).is_some() {
            // Allocate a new handle
            let handle = self.next_handle;
            self.next_handle += 1;
            
            self.handle_to_path.insert(handle, device_path.clone());
            self.path_to_handle.insert(device_path.clone(), handle);
            
            tracing::info!("  -> Allocated handle 0x{:08X} for {}", handle, device_path);
            Ok(vec![handle])
        } else {
            tracing::warn!("  -> Device not found: {}", device_path);
            Ok(vec![u32::MAX]) // -1 = not found
        }
    }
    
    fn package_to_path(&self, _device_tree: &DeviceTree, _args: &[u32]) -> Result<Vec<u32>> {
        Ok(vec![u32::MAX])
    }
    
    fn claim(&mut self, args: &[u32]) -> Result<Vec<u32>> {
        // args: [virt, size, align]
        // returns: [base_addr]
        if args.len() < 3 {
            return Ok(vec![u32::MAX]);
        }
        
        let virt = args[0];
        let size = args[1];
        let _align = args[2];
        
        tracing::debug!("claim: virt=0x{:08X}, size=0x{:08X}", virt, size);
        
        // If virt is non-zero, return it (specific address requested)
        // Otherwise allocate from a pool
        if virt != 0 {
            Ok(vec![virt])
        } else {
            // TODO: Implement proper memory allocation
            Ok(vec![0x00400000]) // Dummy address
        }
    }
    
    fn release(&mut self, _args: &[u32]) -> Result<Vec<u32>> {
        Ok(vec![0])
    }
    
    fn open(&mut self, _args: &[u32]) -> Result<Vec<u32>> {
        // Return a dummy instance handle
        Ok(vec![1])
    }
    
    fn close(&mut self, _args: &[u32]) -> Result<Vec<u32>> {
        Ok(vec![0])
    }
    
    fn read(&mut self, _args: &[u32]) -> Result<Vec<u32>> {
        Ok(vec![0])
    }
    
    fn write(&mut self, args: &[u32]) -> Result<Vec<u32>> {
        // args: [ihandle, buf_ptr, len]
        // returns: [actual_len]
        if args.len() < 3 {
            return Ok(vec![u32::MAX]);
        }
        
        let len = args[2];
        tracing::debug!("write: len={}", len);
        
        // Just claim we wrote it all
        Ok(vec![len])
    }
    
    fn seek(&mut self, _args: &[u32]) -> Result<Vec<u32>> {
        Ok(vec![0])
    }
    
    fn exit(&self) -> Result<Vec<u32>> {
        tracing::info!("OpenFirmware exit called");
        Ok(vec![0])
    }
    
    fn test(&self, args: &[u32]) -> Result<Vec<u32>> {
        // args: [service_name_ptr]
        // returns: [exists] (0 = exists, -1 = doesn't exist)
        tracing::debug!("test service");
        Ok(vec![0]) // Claim all services exist for now
    }
}

impl Default for ClientInterface {
    fn default() -> Self {
        Self::new()
    }
}
