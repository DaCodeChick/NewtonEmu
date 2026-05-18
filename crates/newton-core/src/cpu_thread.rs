// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! CPU thread management
//!
//! This module implements CPU thread separation, allowing the CPU emulation
//! to run on a dedicated thread while communicating with the main thread
//! via message-passing channels.

use newton_cpu::{Cpu, Registers};
use newton_utils::Result;
use std::sync::mpsc::{self, Sender, Receiver, TryRecvError};
use std::thread::{self, JoinHandle};

/// Commands sent to the CPU thread
#[derive(Debug, Clone)]
pub enum CpuCommand {
    /// Start running continuously
    Run,
    
    /// Pause execution
    Pause,
    
    /// Execute a single instruction
    Step,
    
    /// Reset the CPU
    Reset,
    
    /// Run for a specific number of cycles
    RunCycles(u64),
    
    /// Set a breakpoint at an address
    SetBreakpoint(u32),
    
    /// Clear a breakpoint at an address
    ClearBreakpoint(u32),
    
    /// Clear all breakpoints
    ClearAllBreakpoints,
    
    /// Request CPU state snapshot
    GetState,
    
    /// Set CPU register value
    SetRegister { index: usize, value: u32 },
    
    /// Set program counter
    SetPC(u32),
    
    /// Shutdown the CPU thread
    Shutdown,
}

/// Events sent from the CPU thread
#[derive(Debug, Clone)]
pub enum CpuEvent {
    /// CPU state changed (running/paused)
    StateChanged(CpuState),
    
    /// Hit a breakpoint
    Breakpoint(u32),
    
    /// CPU error occurred
    Error(String),
    
    /// Completed requested number of cycles
    CyclesCompleted(u64),
    
    /// CPU state snapshot (in response to GetState)
    State(CpuStateSnapshot),
    
    /// Single step completed
    StepCompleted,
    
    /// CPU was reset
    Reset,
}

/// CPU execution state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuState {
    /// CPU is running
    Running,
    
    /// CPU is paused
    Paused,
    
    /// CPU thread has stopped
    Stopped,
}

/// Snapshot of CPU state for debugging
#[derive(Debug, Clone)]
pub struct CpuStateSnapshot {
    /// General purpose registers
    pub gpr: [u32; 32],
    
    /// Program counter
    pub pc: u32,
    
    /// Link register
    pub lr: u32,
    
    /// Count register
    pub ctr: u32,
    
    /// Condition register
    pub cr: u32,
    
    /// XER register
    pub xer: u32,
    
    /// Current execution state
    pub state: CpuState,
    
    /// Total instructions executed
    pub instruction_count: u64,
}

impl CpuStateSnapshot {
    /// Create snapshot from CPU registers
    pub fn from_registers(regs: &Registers, state: CpuState, count: u64) -> Self {
        Self {
            gpr: regs.gpr,
            pc: regs.pc,
            lr: regs.lr,
            ctr: regs.ctr,
            cr: regs.cr.bits(),
            xer: regs.xer.bits(),
            state,
            instruction_count: count,
        }
    }
}

/// CPU thread handle
///
/// This structure manages the CPU thread lifecycle and provides
/// communication channels for sending commands and receiving events.
///
/// **Note**: JIT compilation is not available in multi-threaded mode
/// because Cranelift's JITModule is not Send. The CPU will run in
/// interpreter-only mode when using this thread.
pub struct CpuThread {
    /// Command sender (to CPU thread)
    command_tx: Sender<CpuCommand>,
    
    /// Event receiver (from CPU thread)
    event_rx: Receiver<CpuEvent>,
    
    /// Thread handle
    thread_handle: Option<JoinHandle<()>>,
    
    /// Current CPU state
    state: CpuState,
}

impl CpuThread {
    /// Create a new CPU thread
    ///
    /// **Note**: The CPU must NOT have JIT enabled (use `Cpu::new()`, not `Cpu::new_with_jit()`)
    /// because Cranelift's JIT is not Send-safe.
    pub fn new(cpu: Cpu, memory: std::sync::Arc<crate::memory::Memory>) -> Self {
        let (command_tx, command_rx) = mpsc::channel();
        let (event_tx, event_rx) = mpsc::channel();
        
        // Spawn CPU thread
        let thread_handle = thread::Builder::new()
            .name("newton-cpu".to_string())
            .spawn(move || {
                cpu_thread_main(cpu, memory, command_rx, event_tx);
            })
            .expect("Failed to spawn CPU thread");
        
        Self {
            command_tx,
            event_rx,
            thread_handle: Some(thread_handle),
            state: CpuState::Paused,
        }
    }
    
    /// Send a command to the CPU thread
    pub fn send_command(&self, cmd: CpuCommand) -> Result<()> {
        self.command_tx
            .send(cmd)
            .map_err(|e| newton_utils::Error::Cpu(format!("Failed to send command: {}", e)))
    }
    
    /// Try to receive an event (non-blocking)
    pub fn try_recv_event(&mut self) -> Option<CpuEvent> {
        match self.event_rx.try_recv() {
            Ok(event) => {
                // Update state tracking
                if let CpuEvent::StateChanged(new_state) = event {
                    self.state = new_state;
                }
                Some(event)
            }
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                self.state = CpuState::Stopped;
                None
            }
        }
    }
    
    /// Receive an event (blocking)
    pub fn recv_event(&mut self) -> Option<CpuEvent> {
        match self.event_rx.recv() {
            Ok(event) => {
                if let CpuEvent::StateChanged(new_state) = event {
                    self.state = new_state;
                }
                Some(event)
            }
            Err(_) => {
                self.state = CpuState::Stopped;
                None
            }
        }
    }
    
    /// Get current CPU state
    pub fn state(&self) -> CpuState {
        self.state
    }
    
    /// Shutdown the CPU thread gracefully
    pub fn shutdown(&mut self) -> Result<()> {
        // Send shutdown command
        let _ = self.send_command(CpuCommand::Shutdown);
        
        // Wait for thread to finish
        if let Some(handle) = self.thread_handle.take() {
            handle
                .join()
                .map_err(|_| newton_utils::Error::Cpu("CPU thread panicked".to_string()))?;
        }
        
        self.state = CpuState::Stopped;
        Ok(())
    }
}

impl Drop for CpuThread {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

/// Main CPU thread function
fn cpu_thread_main(
    mut cpu: Cpu,
    memory: std::sync::Arc<crate::memory::Memory>,
    command_rx: Receiver<CpuCommand>,
    event_tx: Sender<CpuEvent>,
) {
    tracing::info!("CPU thread started");
    
    let mut state = CpuState::Paused;
    let mut breakpoints = std::collections::HashSet::new();
    let mut instruction_count: u64 = 0;
    
    // Send initial state
    let _ = event_tx.send(CpuEvent::StateChanged(state));
    
    loop {
        // Check for commands (non-blocking when running, blocking when paused)
        let command = if state == CpuState::Running {
            match command_rx.try_recv() {
                Ok(cmd) => Some(cmd),
                Err(TryRecvError::Empty) => None,
                Err(TryRecvError::Disconnected) => break,
            }
        } else {
            // Paused: block waiting for command
            match command_rx.recv() {
                Ok(cmd) => Some(cmd),
                Err(_) => break,
            }
        };
        
        // Process command if received
        if let Some(cmd) = command {
            match cmd {
                CpuCommand::Run => {
                    state = CpuState::Running;
                    let _ = event_tx.send(CpuEvent::StateChanged(state));
                }
                
                CpuCommand::Pause => {
                    state = CpuState::Paused;
                    let _ = event_tx.send(CpuEvent::StateChanged(state));
                }
                
                CpuCommand::Step => {
                    // Execute single instruction
                    if let Err(e) = cpu.step(&*memory) {
                        let _ = event_tx.send(CpuEvent::Error(format!("Step failed: {}", e)));
                    } else {
                        instruction_count += 1;
                        let _ = event_tx.send(CpuEvent::StepCompleted);
                        
                        // Check for breakpoint
                        if breakpoints.contains(&cpu.registers.pc) {
                            let _ = event_tx.send(CpuEvent::Breakpoint(cpu.registers.pc));
                        }
                    }
                }
                
                CpuCommand::Reset => {
                    cpu.reset();
                    instruction_count = 0;
                    state = CpuState::Paused;
                    let _ = event_tx.send(CpuEvent::Reset);
                    let _ = event_tx.send(CpuEvent::StateChanged(state));
                }
                
                CpuCommand::RunCycles(cycles) => {
                    for _ in 0..cycles {
                        if let Err(e) = cpu.step(&*memory) {
                            let _ = event_tx.send(CpuEvent::Error(format!("Execution failed: {}", e)));
                            break;
                        }
                        instruction_count += 1;
                        
                        // Check for breakpoint
                        if breakpoints.contains(&cpu.registers.pc) {
                            let _ = event_tx.send(CpuEvent::Breakpoint(cpu.registers.pc));
                            state = CpuState::Paused;
                            let _ = event_tx.send(CpuEvent::StateChanged(state));
                            break;
                        }
                    }
                    let _ = event_tx.send(CpuEvent::CyclesCompleted(cycles));
                }
                
                CpuCommand::SetBreakpoint(addr) => {
                    breakpoints.insert(addr);
                }
                
                CpuCommand::ClearBreakpoint(addr) => {
                    breakpoints.remove(&addr);
                }
                
                CpuCommand::ClearAllBreakpoints => {
                    breakpoints.clear();
                }
                
                CpuCommand::GetState => {
                    let snapshot = CpuStateSnapshot::from_registers(
                        &cpu.registers,
                        state,
                        instruction_count,
                    );
                    let _ = event_tx.send(CpuEvent::State(snapshot));
                }
                
                CpuCommand::SetRegister { index, value } => {
                    if index < 32 {
                        cpu.registers.gpr[index] = value;
                    }
                }
                
                CpuCommand::SetPC(pc) => {
                    cpu.registers.pc = pc;
                }
                
                CpuCommand::Shutdown => {
                    state = CpuState::Stopped;
                    let _ = event_tx.send(CpuEvent::StateChanged(state));
                    break;
                }
            }
        }
        
        // Execute instruction if running
        if state == CpuState::Running {
            match cpu.step(&*memory) {
                Ok(()) => {
                    instruction_count += 1;
                    
                    // Check for breakpoint
                    if breakpoints.contains(&cpu.registers.pc) {
                        let _ = event_tx.send(CpuEvent::Breakpoint(cpu.registers.pc));
                        state = CpuState::Paused;
                        let _ = event_tx.send(CpuEvent::StateChanged(state));
                    }
                }
                Err(e) => {
                    let _ = event_tx.send(CpuEvent::Error(format!("Execution error: {}", e)));
                    state = CpuState::Paused;
                    let _ = event_tx.send(CpuEvent::StateChanged(state));
                }
            }
        }
    }
    
    tracing::info!("CPU thread stopped (executed {} instructions)", instruction_count);
}
