# Threading & Async Architecture for NewtonEmu

## Overview

NewtonEmu needs to support multi-threading and asynchronicity for maximum performance and responsiveness. This document outlines the threading model and how async operations will be integrated.

## Goals

1. **CPU emulation in dedicated thread** - Main emulation loop doesn't block UI
2. **Thread-safe device access** - Peripherals can be accessed from multiple threads
3. **Async I/O** - Network, disk operations don't block emulation
4. **Lock-free where possible** - Minimize contention and maximize performance
5. **Optional parallelism** - User can choose single-threaded or multi-threaded mode

## Architecture

### Thread Model

```
┌─────────────────────────────────────────────────────────────┐
│                         Main Thread                          │
│  - Window management (wgpu display + egui debug)            │
│  - Event handling (keyboard, mouse input)                    │
│  - Configuration UI                                          │
└──────────────────┬──────────────────────────────────────────┘
                   │
                   │ Arc<RwLock<EmulatorState>>
                   │ mpsc channels for commands/events
                   │
┌──────────────────▼──────────────────────────────────────────┐
│                      CPU Thread                              │
│  - CPU emulation loop (interpreter + JIT)                   │
│  - Register state (thread-local)                            │
│  - Memory access (via Arc<RwLock<Memory>>)                  │
│  - Device callbacks (via message passing)                   │
└──────────────────┬──────────────────────────────────────────┘
                   │
                   │ mpsc channels
                   │
┌──────────────────▼──────────────────────────────────────────┐
│                    Device Threads (Optional)                 │
│  - Video rendering (frame buffer updates)                   │
│  - Sound processing (audio buffer generation)               │
│  - Network I/O (async Ethernet/WiFi)                        │
│  - Disk I/O (async file operations)                         │
└─────────────────────────────────────────────────────────────┘
```

### Core Components Thread Safety

#### 1. CPU (`crates/newton-cpu`)

**Current**: Not thread-safe (mutable state)

**Proposed**:
```rust
pub struct Cpu {
    // These are thread-local to the CPU thread
    registers: Registers,          // Not Send/Sync - CPU-owned
    interpreter: Interpreter,      // Immutable, can be Sync
    jit: Option<JITCompiler>,     // Not Send/Sync - CPU-owned
    execution_mode: ExecutionMode,
}

// CPU is NOT Send/Sync - it's pinned to its thread
impl !Send for Cpu {}
impl !Sync for Cpu {}
```

**Rationale**: The CPU state should never be accessed from multiple threads. It runs in a dedicated thread and communicates via channels.

#### 2. Memory (`crates/newton-core/src/memory`)

**Current**: Requires &mut for access

**Proposed**:
```rust
use std::sync::{Arc, RwLock};
use parking_lot::RwLock as FastRwLock; // Faster alternative

pub struct Memory {
    // RAM is the most frequently accessed - use parking_lot for speed
    ram: FastRwLock<Vec<u8>>,
    
    // ROM is read-only after initialization
    rom: Vec<u8>,  // No lock needed!
    
    // MMIO requires synchronization
    mmio_devices: FastRwLock<HashMap<u32, Box<dyn MmioDevice + Send + Sync>>>,
}

pub trait MemoryInterface {
    fn read_u8(&self, addr: u32) -> Result<u8>;   // Changed from &mut
    fn read_u16(&self, addr: u32) -> Result<u16>; // Changed from &mut
    fn read_u32(&self, addr: u32) -> Result<u32>; // Changed from &mut
    
    fn write_u8(&self, addr: u32, value: u8) -> Result<()>;   // Changed from &mut
    fn write_u16(&self, addr: u32, value: u16) -> Result<()>; // Changed from &mut
    fn write_u32(&self, addr: u32, value: u32) -> Result<()>; // Changed from &mut
}

impl MemoryInterface for Memory {
    fn read_u8(&self, addr: u32) -> Result<u8> {
        if addr < ROM_SIZE {
            // Fast path - no lock needed
            Ok(self.rom[addr as usize])
        } else if addr < RAM_END {
            // RAM read - acquire read lock
            let ram = self.ram.read();
            Ok(ram[(addr - RAM_START) as usize])
        } else {
            // MMIO - may need device lock
            let devices = self.mmio_devices.read();
            // ... device access
        }
    }
    
    fn write_u8(&self, addr: u32, value: u8) -> Result<()> {
        if addr < ROM_SIZE {
            // ROM is read-only - error or ignore
            return Err(Error::WriteToRom(addr));
        } else if addr < RAM_END {
            // RAM write - acquire write lock
            let mut ram = self.ram.write();
            ram[(addr - RAM_START) as usize] = value;
            Ok(())
        } else {
            // MMIO - device write
            let devices = self.mmio_devices.write();
            // ... device access
        }
    }
}

// Memory IS Send + Sync - can be shared between threads
unsafe impl Send for Memory {}
unsafe impl Sync for Memory {}
```

**Rationale**: Memory needs to be accessed from CPU thread (reads/writes) and potentially from device threads (DMA, interrupts). Using `RwLock` allows concurrent reads (common case) while serializing writes.

#### 3. Devices (`crates/newton-devices`)

**Current**: Not designed for concurrency

**Proposed**:
```rust
pub trait MmioDevice: Send + Sync {
    fn read(&self, offset: u32) -> Result<u32>;
    fn write(&self, offset: u32, value: u32) -> Result<()>;
}

// Example: Framebuffer
pub struct Framebuffer {
    // Use a triple-buffer for lock-free rendering
    buffers: Arc<TripleBuffer<Vec<u8>>>,
    
    // Configuration is rarely changed
    config: RwLock<FramebufferConfig>,
}

impl MmioDevice for Framebuffer {
    fn read(&self, offset: u32) -> Result<u32> {
        // Reading from current buffer
        let buffer = self.buffers.read();
        // ... read logic
    }
    
    fn write(&self, offset: u32, value: u32) -> Result<()> {
        // Writing to current buffer
        let mut buffer = self.buffers.write();
        // ... write logic
        Ok(())
    }
}
```

**Rationale**: Devices need to be `Send + Sync` to be accessed from multiple threads. Using lock-free data structures (like triple-buffer for video) minimizes contention.

### Communication Channels

#### CPU Thread ← Main Thread

```rust
pub enum CpuCommand {
    Run,
    Pause,
    Step,
    Reset,
    Stop,
    SetExecutionMode(ExecutionMode),
}

// Main thread → CPU thread
let (cmd_tx, cmd_rx) = mpsc::channel::<CpuCommand>();
```

#### CPU Thread → Main Thread

```rust
pub enum CpuEvent {
    StateChanged(CpuState),
    Breakpoint(u32),
    Error(String),
    Stats(EmulationStats),
}

// CPU thread → Main thread
let (event_tx, event_rx) = mpsc::channel::<CpuEvent>();
```

### Async I/O Integration

For I/O-heavy operations (network, disk), use async Rust with Tokio:

```rust
use tokio::runtime::Runtime;

pub struct AsyncDeviceManager {
    runtime: Runtime,
}

impl AsyncDeviceManager {
    pub fn new() -> Self {
        Self {
            runtime: Runtime::new().unwrap(),
        }
    }
    
    pub fn spawn_network_device(&self, device: impl NetworkDevice + Send + 'static) {
        self.runtime.spawn(async move {
            device.run().await;
        });
    }
    
    pub fn spawn_disk_device(&self, device: impl DiskDevice + Send + 'static) {
        self.runtime.spawn(async move {
            device.run().await;
        });
    }
}

// Example: Async network device
pub struct EthernetDevice {
    // ... fields
}

impl NetworkDevice for EthernetDevice {
    async fn run(&self) {
        loop {
            tokio::select! {
                packet = self.receive_packet() => {
                    // Process incoming packet
                }
                () = self.send_queue.notified() => {
                    // Send outgoing packets
                }
            }
        }
    }
}
```

## Implementation Plan

### Phase 1: Make Memory Thread-Safe ✅ (Priority: High)

1. Change `MemoryInterface` to use `&self` instead of `&mut self`
2. Add `RwLock` to `Memory` fields
3. Update all memory access to use locks
4. Benchmark performance impact

**Estimated effort**: 2-4 hours

### Phase 2: CPU Thread Separation (Priority: High)

1. Create `CpuThread` wrapper
2. Implement command/event channels
3. Move CPU loop to dedicated thread
4. Update UI to communicate via channels

**Estimated effort**: 4-6 hours

### Phase 3: Device Thread Safety (Priority: Medium)

1. Make all devices `Send + Sync`
2. Implement lock-free video buffer
3. Add async I/O for network/disk
4. Benchmark and optimize

**Estimated effort**: 6-8 hours

### Phase 4: Optional Parallelism (Priority: Low)

1. Add multi-core JIT compilation
2. Parallel device processing
3. Lock-free data structures
4. Performance tuning

**Estimated effort**: 8-12 hours

## Performance Considerations

### Lock Contention

**Problem**: Frequent memory access could cause lock contention

**Solutions**:
1. Use `parking_lot::RwLock` - faster than std RwLock
2. Cache frequently accessed memory pages in CPU thread
3. Use lock-free algorithms where possible
4. Profile and optimize hot paths

### JIT Compilation Thread Safety

**Current**: JIT compiler is not thread-safe

**Options**:
1. **Single-threaded JIT**: Keep JIT in CPU thread (simplest)
2. **Background compilation**: Compile in separate thread, install atomically
3. **Lock-free code cache**: Use atomic swaps for code pointer updates

**Recommendation**: Start with single-threaded, optimize later if needed.

### Memory Access Patterns

**Observation**: PowerPC programs typically access memory sequentially

**Optimization**: Implement a "fast path" for sequential access:
```rust
impl Memory {
    // Fast path for sequential access (no lock)
    fn read_sequential(&self, addr: u32, buf: &mut [u8]) -> Result<()> {
        // Check if entire range is in ROM
        if addr + buf.len() as u32 <= ROM_SIZE {
            buf.copy_from_slice(&self.rom[addr as usize..]);
            return Ok(());
        }
        
        // Otherwise fall back to locked access
        self.read_locked(addr, buf)
    }
}
```

## Testing Strategy

### Unit Tests

- Test all devices with concurrent access
- Verify memory consistency under concurrent reads/writes
- Test command/event channels

### Integration Tests

- Run CPU thread for N cycles, verify state
- Test pause/resume functionality
- Verify UI updates correctly

### Stress Tests

- High-frequency memory access from multiple threads
- Rapid command sending
- Device interrupts during CPU execution

## Open Questions

1. **Should JIT be thread-safe?** 
   - Pro: Enables background compilation
   - Con: Adds complexity, may not be needed
   - **Decision**: Start single-threaded, optimize later

2. **How to handle interrupts?**
   - Option A: Poll in CPU loop
   - Option B: Interrupt channel
   - **Recommendation**: Interrupt channel for lower latency

3. **Lock granularity for devices?**
   - Option A: One lock per device
   - Option B: Fine-grained locks within devices
   - **Recommendation**: Start with per-device, refine as needed

## Conclusion

The proposed architecture provides:
- ✅ Clean separation of concerns
- ✅ Good performance characteristics
- ✅ Flexibility for future optimization
- ✅ Testable components
- ✅ Optional multi-threading (user choice)

Next steps:
1. Implement Phase 1 (thread-safe memory)
2. Benchmark performance
3. Proceed with Phase 2 if performance is acceptable
