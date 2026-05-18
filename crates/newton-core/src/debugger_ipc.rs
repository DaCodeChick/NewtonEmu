// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Debugger IPC handler for processing commands from frontend

use crate::{DebuggerCommand, DebuggerResponse, Emulator, CpuStateDto};
use std::io::{self, BufRead, Write};
use std::sync::mpsc::{channel, Sender, Receiver, TryRecvError};
use std::thread;

/// Debugger IPC handler
/// 
/// Handles JSON-based commands from stdin and sends responses to stdout.
/// Runs in a background thread to avoid blocking emulation.
pub struct DebuggerIpc {
    command_rx: Receiver<DebuggerCommand>,
    _input_thread: thread::JoinHandle<()>,
}

impl DebuggerIpc {
    /// Start the debugger IPC handler
    /// 
    /// Spawns a thread that reads JSON commands from stdin.
    pub fn start() -> io::Result<Self> {
        let (command_tx, command_rx) = channel();
        
        let input_thread = thread::Builder::new()
            .name("debugger-ipc".to_string())
            .spawn(move || {
                Self::input_loop(command_tx);
            })?;
        
        tracing::info!("Debugger IPC handler started");
        
        Ok(Self {
            command_rx,
            _input_thread: input_thread,
        })
    }
    
    /// Input loop: read commands from stdin and send to channel
    fn input_loop(tx: Sender<DebuggerCommand>) {
        let stdin = io::stdin();
        let reader = stdin.lock();
        
        for line in reader.lines() {
            match line {
                Ok(line) => {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }
                    
                    // Parse JSON command
                    match serde_json::from_str::<DebuggerCommand>(&line) {
                        Ok(cmd) => {
                            if tx.send(cmd).is_err() {
                                tracing::error!("Failed to send command - channel closed");
                                break;
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Failed to parse debugger command: {}", e);
                            Self::send_error(format!("Invalid JSON: {}", e));
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to read from stdin: {}", e);
                    break;
                }
            }
        }
        
        tracing::info!("Debugger IPC input loop exited");
    }
    
    /// Send a response to stdout
    fn send_response(response: DebuggerResponse) {
        match serde_json::to_string(&response) {
            Ok(json) => {
                if let Err(e) = writeln!(io::stdout(), "{}", json) {
                    tracing::error!("Failed to write response: {}", e);
                }
                let _ = io::stdout().flush();
            }
            Err(e) => {
                tracing::error!("Failed to serialize response: {}", e);
            }
        }
    }
    
    /// Send an error response
    fn send_error(message: String) {
        Self::send_response(DebuggerResponse::Error { message });
    }
    
    /// Process pending commands
    /// 
    /// Should be called from the main emulation loop periodically.
    pub fn process_commands(&mut self, emulator: &mut Emulator) {
        loop {
            match self.command_rx.try_recv() {
                Ok(cmd) => {
                    self.handle_command(cmd, emulator);
                }
                Err(TryRecvError::Empty) => {
                    // No more commands
                    break;
                }
                Err(TryRecvError::Disconnected) => {
                    tracing::warn!("Debugger IPC channel disconnected");
                    break;
                }
            }
        }
    }
    
    /// Handle a single command
    fn handle_command(&self, cmd: DebuggerCommand, emulator: &mut Emulator) {
        use DebuggerCommand::*;
        
        let response = match cmd {
            Enable => {
                emulator.enable_debugger();
                DebuggerResponse::Ok
            }
            
            Disable => {
                emulator.disable_debugger();
                DebuggerResponse::Ok
            }
            
            Pause => {
                if let Some(debugger) = emulator.debugger_mut() {
                    debugger.pause();
                    DebuggerResponse::Ok
                } else {
                    DebuggerResponse::Error {
                        message: "Debugger not enabled".to_string()
                    }
                }
            }
            
            Resume => {
                if let Some(debugger) = emulator.debugger_mut() {
                    debugger.resume();
                    DebuggerResponse::Ok
                } else {
                    DebuggerResponse::Error {
                        message: "Debugger not enabled".to_string()
                    }
                }
            }
            
            StepInto => {
                if let Some(debugger) = emulator.debugger_mut() {
                    debugger.step_into();
                    DebuggerResponse::Ok
                } else {
                    DebuggerResponse::Error {
                        message: "Debugger not enabled".to_string()
                    }
                }
            }
            
            StepOver => {
                // Get current PC from CPU first
                let pc = if let Some(cpu) = emulator.cpu() {
                    cpu.registers.pc
                } else {
                    0
                };
                
                if let Some(debugger) = emulator.debugger_mut() {
                    debugger.step_over(pc);
                    DebuggerResponse::Ok
                } else {
                    DebuggerResponse::Error {
                        message: "Debugger not enabled".to_string()
                    }
                }
            }
            
            StepOut => {
                if let Some(debugger) = emulator.debugger_mut() {
                    debugger.step_out();
                    DebuggerResponse::Ok
                } else {
                    DebuggerResponse::Error {
                        message: "Debugger not enabled".to_string()
                    }
                }
            }
            
            AddBreakpoint { address, bp_type } => {
                if let Some(debugger) = emulator.debugger_mut() {
                    debugger.add_breakpoint(address, bp_type.into());
                    DebuggerResponse::Ok
                } else {
                    DebuggerResponse::Error {
                        message: "Debugger not enabled".to_string()
                    }
                }
            }
            
            RemoveBreakpoint { address, bp_type } => {
                if let Some(debugger) = emulator.debugger_mut() {
                    debugger.remove_breakpoint(address, bp_type.into());
                    DebuggerResponse::Ok
                } else {
                    DebuggerResponse::Error {
                        message: "Debugger not enabled".to_string()
                    }
                }
            }
            
            GetCpuState => {
                if let Some(cpu) = emulator.cpu() {
                    let exec_state = if let Some(debugger) = emulator.debugger() {
                        debugger.state().into()
                    } else {
                        crate::debugger_protocol::ExecutionStateDto::Running
                    };
                    
                    let instruction_count = emulator.debugger()
                        .map(|d| d.instruction_count())
                        .unwrap_or(0);
                    
                    DebuggerResponse::CpuState {
                        state: CpuStateDto {
                            pc: cpu.registers.pc,
                            lr: cpu.registers.lr,
                            ctr: cpu.registers.ctr,
                            cr: cpu.registers.cr.bits(),
                            xer: cpu.registers.xer.bits(),
                            gpr: cpu.registers.gpr,
                            fpr: cpu.registers.fpr,
                            execution_state: exec_state,
                            instruction_count,
                        }
                    }
                } else {
                    DebuggerResponse::Error {
                        message: "CPU not available".to_string()
                    }
                }
            }
            
            GetMemory { address: _, length: _ } => {
                // Memory is mmapped - frontend should read directly
                DebuggerResponse::Error {
                    message: format!(
                        "Use memory-mapped file instead: {}",
                        emulator.memory().ram_path().display()
                    )
                }
            }
            
            WriteMemory { address: _, data: _ } => {
                // Could implement, but dangerous - prefer mmap writes
                DebuggerResponse::Error {
                    message: "Use memory-mapped file for writes".to_string()
                }
            }
            
            GetBreakpoints => {
                if let Some(debugger) = emulator.debugger() {
                    let breakpoints = debugger.breakpoints()
                        .into_iter()
                        .map(|bp| bp.into())
                        .collect();
                    DebuggerResponse::Breakpoints { breakpoints }
                } else {
                    DebuggerResponse::Error {
                        message: "Debugger not enabled".to_string()
                    }
                }
            }
            
            Disassemble { address: _, count: _ } => {
                // TODO: Implement PowerPC disassembly
                DebuggerResponse::Error {
                    message: "Disassembly not yet implemented".to_string()
                }
            }
        };
        
        Self::send_response(response);
    }
}
