// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! CHRP boot script parser and executor
//!
//! CHRP (Common Hardware Reference Platform) boot scripts are Forth-based
//! scripts embedded in NewWorld Mac ROMs. They configure the boot environment
//! and load the operating system.

use newton_utils::Result;

/// CHRP boot script parser
pub struct BootScriptParser {
    script: String,
}

impl BootScriptParser {
    /// Create a new boot script parser
    pub fn new(script: String) -> Self {
        Self { script }
    }
    
    /// Parse the boot script and extract key information
    pub fn parse(&self) -> Result<BootInfo> {
        let mut info = BootInfo::default();
        
        // Parse key fields from the script
        // CHRP scripts typically contain:
        // - boot-device: which device to boot from
        // - boot-file: which file to load
        // - load-base: where to load the file
        
        for line in self.script.lines() {
            let line = line.trim();
            
            // Look for boot-script field
            if line.starts_with("boot-script") {
                // Extract the Forth code between the field markers
                // Format: boot-script  <forth code here>
                if let Some(start) = line.find(char::is_whitespace) {
                    info.boot_script = Some(line[start..].trim().to_string());
                }
            }
            
            // Look for boot-device
            else if line.starts_with("boot-device") {
                if let Some(device) = Self::extract_field_value(line) {
                    info.boot_device = Some(device);
                }
            }
            
            // Look for boot-file
            else if line.starts_with("boot-file") {
                if let Some(file) = Self::extract_field_value(line) {
                    info.boot_file = Some(file);
                }
            }
            
            // Look for load-base address
            else if line.starts_with("load-base") {
                if let Some(base_str) = Self::extract_field_value(line) {
                    // Try to parse as hex
                    if let Some(hex_str) = base_str.strip_prefix("0x") {
                        if let Ok(addr) = u32::from_str_radix(hex_str, 16) {
                            info.load_base = Some(addr);
                        }
                    }
                }
            }
        }
        
        tracing::info!("Parsed CHRP boot script:");
        if let Some(ref device) = info.boot_device {
            tracing::info!("  boot-device: {}", device);
        }
        if let Some(ref file) = info.boot_file {
            tracing::info!("  boot-file: {}", file);
        }
        if let Some(base) = info.load_base {
            tracing::info!("  load-base: 0x{:08X}", base);
        }
        
        Ok(info)
    }
    
    /// Extract field value from a line like "field-name  value"
    fn extract_field_value(line: &str) -> Option<String> {
        // Split on whitespace and take everything after the field name
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            // Join all parts after the first (field name)
            Some(parts[1..].join(" "))
        } else {
            None
        }
    }
}

/// Parsed boot script information
#[derive(Debug, Default)]
pub struct BootInfo {
    /// Boot device path
    pub boot_device: Option<String>,
    
    /// Boot file path
    pub boot_file: Option<String>,
    
    /// Load base address
    pub load_base: Option<u32>,
    
    /// Forth boot script code
    pub boot_script: Option<String>,
}

impl BootInfo {
    /// Check if we have enough information to attempt a boot
    pub fn is_bootable(&self) -> bool {
        self.boot_device.is_some() || self.boot_file.is_some()
    }
}
