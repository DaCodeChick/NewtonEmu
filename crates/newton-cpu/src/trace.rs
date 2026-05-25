// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Execution tracing for debugging and analysis

use std::collections::VecDeque;

/// A single execution trace entry
#[derive(Debug, Clone)]
pub struct TraceEntry {
    /// Program counter
    pub pc: u32,
    /// Instruction opcode
    pub instruction: u32,
    /// Link register value
    pub lr: u32,
    /// Stack pointer (r1)
    pub sp: u32,
    /// TOC pointer (r2)
    pub r2: u32,
    /// Condition register
    pub cr: u32,
}

/// Ring buffer for execution trace
pub struct TraceBuffer {
    /// Circular buffer of trace entries
    entries: VecDeque<TraceEntry>,
    /// Maximum number of entries to keep
    capacity: usize,
    /// Whether tracing is enabled
    enabled: bool,
}

impl TraceBuffer {
    /// Create a new trace buffer with the given capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: VecDeque::with_capacity(capacity),
            capacity,
            enabled: false,
        }
    }
    
    /// Enable tracing
    pub fn enable(&mut self) {
        self.enabled = true;
    }
    
    /// Disable tracing
    pub fn disable(&mut self) {
        self.enabled = false;
    }
    
    /// Check if tracing is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    /// Add a trace entry
    pub fn record(&mut self, entry: TraceEntry) {
        if !self.enabled {
            return;
        }
        
        if self.entries.len() >= self.capacity {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
    }
    
    /// Get all trace entries (oldest first)
    pub fn entries(&self) -> &VecDeque<TraceEntry> {
        &self.entries
    }
    
    /// Get the last N entries
    pub fn last_n(&self, n: usize) -> Vec<&TraceEntry> {
        let start = self.entries.len().saturating_sub(n);
        self.entries.iter().skip(start).collect()
    }
    
    /// Clear all entries
    pub fn clear(&mut self) {
        self.entries.clear();
    }
    
    /// Print the trace buffer to stderr
    pub fn dump(&self) {
        eprintln!("\n========== Execution Trace (last {} instructions) ==========", self.entries.len());
        for (i, entry) in self.entries.iter().enumerate() {
            eprintln!(
                "{:4}: PC=0x{:08X}  inst=0x{:08X}  LR=0x{:08X}  SP=0x{:08X}  r2=0x{:08X}  CR=0x{:08X}",
                i, entry.pc, entry.instruction, entry.lr, entry.sp, entry.r2, entry.cr
            );
        }
        eprintln!("===========================================================\n");
    }
    
    /// Print the last N entries
    pub fn dump_last_n(&self, n: usize) {
        let entries = self.last_n(n);
        eprintln!("\n========== Execution Trace (last {} instructions) ==========", entries.len());
        for (i, entry) in entries.iter().enumerate() {
            eprintln!(
                "{:4}: PC=0x{:08X}  inst=0x{:08X}  LR=0x{:08X}  SP=0x{:08X}  r2=0x{:08X}  CR=0x{:08X}",
                i, entry.pc, entry.instruction, entry.lr, entry.sp, entry.r2, entry.cr
            );
        }
        eprintln!("===========================================================\n");
    }
}

impl Default for TraceBuffer {
    fn default() -> Self {
        // Default to 100 entries
        Self::new(100)
    }
}
