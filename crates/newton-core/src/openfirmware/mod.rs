// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! OpenFirmware (IEEE 1275) implementation
//!
//! Provides device tree, client interface, and boot support for NewWorld ROMs.

pub mod device_tree;
pub mod client_interface;
pub mod boot_script;
pub mod forth;
pub mod forth_of_words;

pub use device_tree::{DeviceTree, DeviceNode, Property};
pub use client_interface::{ClientInterface, ServiceResult};
pub use boot_script::{BootScriptParser, BootInfo};
pub use forth::ForthInterpreter;
pub use forth_of_words::OpenFirmwareForthExt;

/// OpenFirmware state
pub struct OpenFirmware {
    /// Device tree
    device_tree: DeviceTree,
    
    /// Client interface
    client_interface: ClientInterface,
    
    /// Forth interpreter
    forth: ForthInterpreter,
}

impl OpenFirmware {
    /// Create a new OpenFirmware instance
    pub fn new() -> Self {
        let mut of = Self {
            device_tree: DeviceTree::new(),
            client_interface: ClientInterface::new(),
            forth: ForthInterpreter::new(),
        };
        
        // Initialize basic device tree
        of.init_device_tree();
        
        // Register OpenFirmware-specific Forth words
        of.forth.register_of_words();
        
        of
    }
    
    /// Initialize the device tree with standard Mac hardware
    fn init_device_tree(&mut self) {
        tracing::info!("Initializing OpenFirmware device tree");
        
        // Root node
        {
            let root = self.device_tree.root_mut();
            root.add_property("device_type", b"chrp");
            root.add_property("model", b"Power Macintosh");
            root.add_property("compatible", b"iMac,1\0PowerMac1,1\0");
            root.add_property("#address-cells", &1u32.to_be_bytes());
            root.add_property("#size-cells", &1u32.to_be_bytes());
        }
        
        // CPU node
        let mut cpus = DeviceNode::new("cpus", "");
        cpus.add_property("device_type", b"cpu");
        cpus.add_property("#address-cells", &1u32.to_be_bytes());
        cpus.add_property("#size-cells", &0u32.to_be_bytes());
        
        let mut cpu = DeviceNode::new("PowerPC,G4", "");
        cpu.add_property("device_type", b"cpu");
        cpu.add_property("cpu-version", &0x800C0000u32.to_be_bytes()); // PVR for G4
        cpu.add_property("clock-frequency", &500_000_000u32.to_be_bytes()); // 500 MHz
        cpu.add_property("bus-frequency", &100_000_000u32.to_be_bytes()); // 100 MHz bus
        cpu.add_property("timebase-frequency", &25_000_000u32.to_be_bytes()); // 25 MHz
        cpu.add_property("reg", &0u32.to_be_bytes());
        
        self.device_tree.add_child("/cpus", cpu);
        self.device_tree.add_node("/cpus", cpus);
        
        // Memory node
        let mut memory = DeviceNode::new("memory", "");
        memory.add_property("device_type", b"memory");
        let mem_reg = [
            0x00000000u32.to_be_bytes(),
            0x10000000u32.to_be_bytes(), // 256MB
        ].concat();
        memory.add_property("reg", mem_reg);
        self.device_tree.add_node("/memory", memory);
        
        // PCI host bridge
        let mut pci = DeviceNode::new("pci", "");
        pci.add_property("device_type", b"pci");
        pci.add_property("compatible", b"grackle");
        pci.add_property("#address-cells", &3u32.to_be_bytes());
        pci.add_property("#size-cells", &2u32.to_be_bytes());
        pci.add_property("reg", &0xFEC00000u32.to_be_bytes());
        let bus_range = [0u32.to_be_bytes(), 0u32.to_be_bytes()].concat();
        pci.add_property("bus-range", bus_range);
        self.device_tree.add_node("/pci", pci);
        
        // Video display
        let mut display = DeviceNode::new("ATY,RageProVRAM", "");
        display.add_property("device_type", b"display");
        display.add_property("model", b"ATY,Rage128");
        display.add_property("width", &800u32.to_be_bytes());
        display.add_property("height", &600u32.to_be_bytes());
        display.add_property("depth", &32u32.to_be_bytes());
        display.add_property("linebytes", &(800 * 4u32).to_be_bytes());
        self.device_tree.add_node("/pci/ATY,RageProVRAM", display);
        
        // Chosen node (runtime properties)
        let mut chosen = DeviceNode::new("chosen", "");
        chosen.add_property("stdin", &0u32.to_be_bytes());
        chosen.add_property("stdout", &0u32.to_be_bytes());
        chosen.add_property("bootargs", b"");
        self.device_tree.add_node("/chosen", chosen);
        
        // Options node (NVRAM settings)
        let mut options = DeviceNode::new("options", "");
        options.add_property("boot-device", b"hd:,\\:tbxi");
        options.add_property("boot-file", b"");
        options.add_property("auto-boot?", b"true");
        self.device_tree.add_node("/options", options);
        
        tracing::info!("OpenFirmware device tree initialized");
    }
    
    /// Get the device tree
    pub fn device_tree(&self) -> &DeviceTree {
        &self.device_tree
    }
    
    /// Get the device tree mutably
    pub fn device_tree_mut(&mut self) -> &mut DeviceTree {
        &mut self.device_tree
    }
    
    /// Get the client interface
    pub fn client_interface(&self) -> &ClientInterface {
        &self.client_interface
    }
    
    /// Get the client interface mutably
    pub fn client_interface_mut(&mut self) -> &mut ClientInterface {
        &mut self.client_interface
    }
    
    /// Get the Forth interpreter
    pub fn forth(&self) -> &ForthInterpreter {
        &self.forth
    }
    
    /// Get the Forth interpreter mutably
    pub fn forth_mut(&mut self) -> &mut ForthInterpreter {
        &mut self.forth
    }
    
    /// Execute Forth code
    pub fn execute_forth(&mut self, code: &str) -> newton_utils::Result<()> {
        self.forth.eval(code)
    }
    
    /// Call a client interface service
    pub fn call_client_service(
        &mut self,
        service: &str,
        args: &[u32],
        string_args: &[String],
    ) -> newton_utils::Result<ServiceResult> {
        self.client_interface.call_service(&mut self.device_tree, service, args, string_args)
    }
}

impl Default for OpenFirmware {
    fn default() -> Self {
        Self::new()
    }
}
