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

/// Memory region allocation
#[derive(Debug, Clone)]
struct MemoryRegion {
    base: u32,
    size: u32,
}

/// Result of a client interface service call
pub struct ServiceResult {
    /// Return values to write back to the return value array
    pub returns: Vec<u32>,
    
    /// Optional memory write (address, data)
    pub memory_write: Option<(u32, Vec<u8>)>,
}

impl ServiceResult {
    /// Create a simple result with just return values
    pub fn new(returns: Vec<u32>) -> Self {
        Self {
            returns,
            memory_write: None,
        }
    }
    
    /// Create a result with return values and a memory write
    pub fn with_memory_write(returns: Vec<u32>, addr: u32, data: Vec<u8>) -> Self {
        Self {
            returns,
            memory_write: Some((addr, data)),
        }
    }
}

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
    
    /// Allocated memory regions
    allocated_regions: Vec<MemoryRegion>,
    
    /// Next available address for dynamic allocation
    next_alloc_addr: u32,
}

impl ClientInterface {
    /// Create a new client interface
    pub fn new() -> Self {
        Self {
            next_handle: 1,
            handle_to_path: HashMap::new(),
            path_to_handle: HashMap::new(),
            allocated_regions: Vec::new(),
            next_alloc_addr: 0x00400000, // Start allocating from 4MB
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
    ) -> Result<ServiceResult> {
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
                Ok(ServiceResult::new(vec![u32::MAX])) // Return failure
            }
        }
    }
    
    fn peer(&mut self, device_tree: &DeviceTree, args: &[u32]) -> Result<ServiceResult> {
        // args: [phandle]
        // returns: [peer_phandle]
        if args.is_empty() {
            return Ok(ServiceResult::new(vec![u32::MAX]));
        }
        
        let phandle = args[0];
        
        // Special case: peer(0) returns first child of root
        if phandle == 0 {
            let children = device_tree.get_children("/");
            if let Some(first_child) = children.first() {
                return self.get_or_allocate_handle(device_tree, first_child);
            } else {
                return Ok(ServiceResult::new(vec![0]));
            }
        }
        
        // Look up the path for this phandle
        if let Some(path) = self.handle_to_path.get(&phandle) {
            let peers = device_tree.get_peers(path);
            
            // Find the current node in the peer list
            if let Some(current_idx) = peers.iter().position(|p| p == path) {
                // Return the next peer (sibling)
                if current_idx + 1 < peers.len() {
                    let next_peer = &peers[current_idx + 1];
                    return self.get_or_allocate_handle(device_tree, next_peer);
                }
            }
        }
        
        // No more peers
        Ok(ServiceResult::new(vec![0]))
    }
    
    fn child(&mut self, device_tree: &DeviceTree, args: &[u32]) -> Result<ServiceResult> {
        // args: [phandle]
        // returns: [child_phandle]
        if args.is_empty() {
            return Ok(ServiceResult::new(vec![u32::MAX]));
        }
        
        let phandle = args[0];
        
        // Look up the path for this phandle
        if let Some(path) = self.handle_to_path.get(&phandle) {
            let children = device_tree.get_children(path);
            
            // Return first child
            if let Some(first_child) = children.first() {
                return self.get_or_allocate_handle(device_tree, first_child);
            }
        }
        
        // No children
        Ok(ServiceResult::new(vec![0]))
    }
    
    fn parent(&mut self, device_tree: &DeviceTree, args: &[u32]) -> Result<ServiceResult> {
        // args: [phandle]
        // returns: [parent_phandle]
        if args.is_empty() {
            return Ok(ServiceResult::new(vec![u32::MAX]));
        }
        
        let phandle = args[0];
        
        // Look up the path for this phandle
        if let Some(path) = self.handle_to_path.get(&phandle) {
            if let Some(parent_path) = device_tree.get_parent_path(path) {
                return self.get_or_allocate_handle(device_tree, &parent_path);
            }
        }
        
        // No parent (root node)
        Ok(ServiceResult::new(vec![0]))
    }
    
    /// Get or allocate a handle for a path
    fn get_or_allocate_handle(&mut self, device_tree: &DeviceTree, path: &str) -> Result<ServiceResult> {
        // Check if we already have a handle
        if let Some(&handle) = self.path_to_handle.get(path) {
            return Ok(ServiceResult::new(vec![handle]));
        }
        
        // Verify the path exists
        if device_tree.find_node(path).is_some() {
            // Allocate new handle
            let handle = self.next_handle;
            self.next_handle += 1;
            
            self.handle_to_path.insert(handle, path.to_string());
            self.path_to_handle.insert(path.to_string(), handle);
            
            tracing::debug!("Allocated handle 0x{:08X} for {}", handle, path);
            Ok(ServiceResult::new(vec![handle]))
        } else {
            Ok(ServiceResult::new(vec![0]))
        }
    }
    
    fn getprop(&self, device_tree: &DeviceTree, args: &[u32], string_args: &[String]) -> Result<ServiceResult> {
        // args: [phandle, property_name_ptr, buf_ptr, buf_len]
        // returns: [actual_len]
        tracing::debug!("getprop: args.len()={}, string_args.len()={}, string_args={:?}", 
                       args.len(), string_args.len(), string_args);
        
        if args.len() < 4 || string_args.len() < 2 {
            tracing::warn!("getprop: insufficient args or string_args");
            return Ok(ServiceResult::new(vec![u32::MAX])); // -1 = error
        }
        
        let phandle = args[0];
        let property_name = &string_args[1];
        let buf_ptr = args[2];
        let buf_len = args[3];
        
        tracing::debug!("getprop: phandle=0x{:08X}, property={}, buf=0x{:08X}, len={}", 
                       phandle, property_name, buf_ptr, buf_len);
        
        // Look up the device path from the handle
        if let Some(path) = self.handle_to_path.get(&phandle) {
            tracing::debug!("  -> Found path '{}' for phandle 0x{:08X}", path, phandle);
            // Get the property value
            if let Some(value) = device_tree.get_property(path, property_name) {
                let len = value.len() as u32;
                
                // Copy as much as will fit in the buffer
                let copy_len = std::cmp::min(len, buf_len);
                let data_to_write = value[..copy_len as usize].to_vec();
                
                tracing::debug!("  -> Found property, len={}, copying {} bytes to 0x{:08X}", 
                               len, copy_len, buf_ptr);
                
                return Ok(ServiceResult::with_memory_write(vec![len], buf_ptr, data_to_write));
            } else {
                tracing::warn!("  -> Property '{}' not found on path '{}'", property_name, path);
            }
        } else {
            tracing::warn!("  -> phandle 0x{:08X} not found in handle_to_path map", phandle);
        }
        
        tracing::warn!("  -> Property not found");
        Ok(ServiceResult::new(vec![u32::MAX])) // -1 = not found
    }
    
    fn getproplen(&self, device_tree: &DeviceTree, args: &[u32], string_args: &[String]) -> Result<ServiceResult> {
        // args: [phandle, property_name_ptr]
        // returns: [len]
        if args.len() < 2 || string_args.len() < 2 {
            return Ok(ServiceResult::new(vec![u32::MAX]));
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
                return Ok(ServiceResult::new(vec![len]));
            }
        }
        
        tracing::warn!("  -> Property not found");
        Ok(ServiceResult::new(vec![u32::MAX])) // -1 = not found
    }
    
    fn nextprop(&self, _device_tree: &DeviceTree, _args: &[u32]) -> Result<ServiceResult> {
        Ok(ServiceResult::new(vec![u32::MAX]))
    }
    
    fn setprop(&mut self, _device_tree: &mut DeviceTree, _args: &[u32]) -> Result<ServiceResult> {
        Ok(ServiceResult::new(vec![0]))
    }
    
    fn finddevice(&mut self, device_tree: &DeviceTree, _args: &[u32], string_args: &[String]) -> Result<ServiceResult> {
        // args: [device_path_ptr]
        // returns: [phandle]
        if string_args.is_empty() {
            return Ok(ServiceResult::new(vec![u32::MAX]));
        }
        
        let device_path = &string_args[0];
        tracing::debug!("finddevice: path={}", device_path);
        
        // Check if we already have a handle for this path
        if let Some(&handle) = self.path_to_handle.get(device_path) {
            tracing::debug!("  -> Found existing handle 0x{:08X}", handle);
            return Ok(ServiceResult::new(vec![handle]));
        }
        
        // Look up the device in the tree
        if device_tree.find_node(device_path).is_some() {
            // Allocate a new handle
            let handle = self.next_handle;
            self.next_handle += 1;
            
            self.handle_to_path.insert(handle, device_path.clone());
            self.path_to_handle.insert(device_path.clone(), handle);
            
            tracing::info!("  -> Allocated handle 0x{:08X} for {}", handle, device_path);
            Ok(ServiceResult::new(vec![handle]))
        } else {
            tracing::warn!("  -> Device not found: {}", device_path);
            Ok(ServiceResult::new(vec![u32::MAX])) // -1 = not found
        }
    }
    
    fn package_to_path(&self, _device_tree: &DeviceTree, _args: &[u32]) -> Result<ServiceResult> {
        Ok(ServiceResult::new(vec![u32::MAX]))
    }
    
    fn claim(&mut self, args: &[u32]) -> Result<ServiceResult> {
        // args: [virt, size, align]
        // returns: [base_addr]
        if args.len() < 3 {
            return Ok(ServiceResult::new(vec![u32::MAX]));
        }
        
        let virt = args[0];
        let size = args[1];
        let align = args[2];
        
        tracing::debug!("claim: virt=0x{:08X}, size=0x{:08X}, align=0x{:08X}", virt, size, align);
        
        let base_addr = if virt != 0 {
            // Specific address requested
            // Check if it overlaps with existing allocations
            for region in &self.allocated_regions {
                let region_end = region.base + region.size;
                let request_end = virt + size;
                
                if (virt >= region.base && virt < region_end) ||
                   (request_end > region.base && request_end <= region_end) ||
                   (virt <= region.base && request_end >= region_end) {
                    tracing::warn!("  -> Address 0x{:08X} overlaps with existing allocation at 0x{:08X}", 
                                  virt, region.base);
                    return Ok(ServiceResult::new(vec![u32::MAX])); // -1 = failed
                }
            }
            virt
        } else {
            // Dynamic allocation
            let mut addr = self.next_alloc_addr;
            
            // Apply alignment
            if align > 0 {
                let mask = align - 1;
                if (addr & mask) != 0 {
                    addr = (addr + align) & !mask;
                }
            }
            
            // Update next allocation address
            self.next_alloc_addr = addr + size;
            
            addr
        };
        
        // Record the allocation
        self.allocated_regions.push(MemoryRegion {
            base: base_addr,
            size,
        });
        
        tracing::info!("  -> Claimed 0x{:08X} bytes at 0x{:08X}", size, base_addr);
        Ok(ServiceResult::new(vec![base_addr]))
    }
    
    fn release(&mut self, args: &[u32]) -> Result<ServiceResult> {
        // args: [virt, size]
        // returns: [result]
        if args.len() < 2 {
            return Ok(ServiceResult::new(vec![u32::MAX]));
        }
        
        let virt = args[0];
        let size = args[1];
        
        tracing::debug!("release: virt=0x{:08X}, size=0x{:08X}", virt, size);
        
        // Find and remove the region
        if let Some(pos) = self.allocated_regions.iter().position(|r| r.base == virt && r.size == size) {
            self.allocated_regions.remove(pos);
            tracing::info!("  -> Released 0x{:08X} bytes at 0x{:08X}", size, virt);
            Ok(ServiceResult::new(vec![0])) // Success
        } else {
            tracing::warn!("  -> Region not found: 0x{:08X} (size 0x{:08X})", virt, size);
            Ok(ServiceResult::new(vec![u32::MAX])) // -1 = not found
        }
    }
    
    fn open(&mut self, _args: &[u32]) -> Result<ServiceResult> {
        // Return a dummy instance handle
        Ok(ServiceResult::new(vec![1]))
    }
    
    fn close(&mut self, _args: &[u32]) -> Result<ServiceResult> {
        Ok(ServiceResult::new(vec![0]))
    }
    
    fn read(&mut self, _args: &[u32]) -> Result<ServiceResult> {
        Ok(ServiceResult::new(vec![0]))
    }
    
    fn write(&mut self, args: &[u32]) -> Result<ServiceResult> {
        // args: [ihandle, buf_ptr, len]
        // returns: [actual_len]
        if args.len() < 3 {
            return Ok(ServiceResult::new(vec![u32::MAX]));
        }
        
        let len = args[2];
        tracing::debug!("write: len={}", len);
        
        // Just claim we wrote it all
        Ok(ServiceResult::new(vec![len]))
    }
    
    fn seek(&mut self, _args: &[u32]) -> Result<ServiceResult> {
        Ok(ServiceResult::new(vec![0]))
    }
    
    fn exit(&self) -> Result<ServiceResult> {
        tracing::info!("OpenFirmware exit called");
        Ok(ServiceResult::new(vec![0]))
    }
    
    fn test(&self, _args: &[u32]) -> Result<ServiceResult> {
        // args: [service_name_ptr]
        // returns: [exists] (0 = exists, -1 = doesn't exist)
        tracing::debug!("test service");
        Ok(ServiceResult::new(vec![0])) // Claim all services exist for now
    }
}

impl Default for ClientInterface {
    fn default() -> Self {
        Self::new()
    }
}
