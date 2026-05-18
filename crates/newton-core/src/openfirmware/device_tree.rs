// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! OpenFirmware device tree

use std::collections::HashMap;

/// A property in the device tree
#[derive(Debug, Clone)]
pub struct Property {
    pub name: String,
    pub value: Vec<u8>,
}

impl Property {
    pub fn new(name: impl Into<String>, value: impl Into<Vec<u8>>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

/// A device node in the device tree
#[derive(Debug, Clone)]
pub struct DeviceNode {
    /// Node name
    pub name: String,
    
    /// Node type (if any)
    pub node_type: String,
    
    /// Properties
    pub properties: HashMap<String, Vec<u8>>,
    
    /// Child nodes
    pub children: Vec<DeviceNode>,
}

impl DeviceNode {
    /// Create a new device node
    pub fn new(name: impl Into<String>, node_type: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            node_type: node_type.into(),
            properties: HashMap::new(),
            children: Vec::new(),
        }
    }
    
    /// Add a property
    pub fn add_property(&mut self, name: impl Into<String>, value: impl Into<Vec<u8>>) {
        self.properties.insert(name.into(), value.into());
    }
    
    /// Get a property value
    pub fn get_property(&self, name: &str) -> Option<&[u8]> {
        self.properties.get(name).map(|v| v.as_slice())
    }
    
    /// Add a child node
    pub fn add_child(&mut self, child: DeviceNode) {
        self.children.push(child);
    }
    
    /// Find a child by name
    pub fn find_child(&self, name: &str) -> Option<&DeviceNode> {
        self.children.iter().find(|c| c.name == name)
    }
    
    /// Find a child by name (mutable)
    pub fn find_child_mut(&mut self, name: &str) -> Option<&mut DeviceNode> {
        self.children.iter_mut().find(|c| c.name == name)
    }
}

/// OpenFirmware device tree
#[derive(Debug, Clone)]
pub struct DeviceTree {
    /// Root node
    root: DeviceNode,
    
    /// Flat map of path -> node for quick lookups
    nodes: HashMap<String, DeviceNode>,
}

impl DeviceTree {
    /// Create a new empty device tree
    pub fn new() -> Self {
        let root = DeviceNode::new("", "");
        Self {
            root,
            nodes: HashMap::new(),
        }
    }
    
    /// Get the root node
    pub fn root(&self) -> &DeviceNode {
        &self.root
    }
    
    /// Get the root node mutably
    pub fn root_mut(&mut self) -> &mut DeviceNode {
        &mut self.root
    }
    
    /// Add a node at a specific path
    pub fn add_node(&mut self, path: impl Into<String>, node: DeviceNode) {
        let path = path.into();
        self.nodes.insert(path, node);
    }
    
    /// Add a child node to a parent path
    pub fn add_child(&mut self, parent_path: &str, child: DeviceNode) {
        let child_path = if parent_path == "/" {
            format!("/{}", child.name)
        } else {
            format!("{}/{}", parent_path, child.name)
        };
        
        self.nodes.insert(child_path, child);
    }
    
    /// Find a node by path
    pub fn find_node(&self, path: &str) -> Option<&DeviceNode> {
        if path == "/" || path.is_empty() {
            Some(&self.root)
        } else {
            self.nodes.get(path)
        }
    }
    
    /// Find a node by path (mutable)
    pub fn find_node_mut(&mut self, path: &str) -> Option<&mut DeviceNode> {
        if path == "/" || path.is_empty() {
            Some(&mut self.root)
        } else {
            self.nodes.get_mut(path)
        }
    }
    
    /// Get a property from a node
    pub fn get_property(&self, path: &str, property: &str) -> Option<&[u8]> {
        self.find_node(path)?.get_property(property)
    }
    
    /// Set a property on a node
    pub fn set_property(&mut self, path: &str, property: impl Into<String>, value: impl Into<Vec<u8>>) {
        if let Some(node) = self.find_node_mut(path) {
            node.add_property(property, value);
        }
    }
}

impl Default for DeviceTree {
    fn default() -> Self {
        Self::new()
    }
}
