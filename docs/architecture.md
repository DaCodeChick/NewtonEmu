# NewtonEmu Architecture

## Overview

NewtonEmu is a PowerPC Macintosh emulator designed for accuracy and performance. This document describes the high-level architecture and design decisions.

## System Components

### CPU Emulation (newton-cpu)

The CPU emulator implements the PowerPC G4 (7400/7450) architecture.

#### Registers
- 32 General Purpose Registers (GPRs): r0-r31
- 32 Floating Point Registers (FPRs): f0-f31  
- 32 Vector Registers (VRs): v0-v31 for AltiVec
- Special registers: PC, LR, CTR, CR, XER, MSR, SRs, SPRs

#### Execution Modes

1. **Interpreter Mode**: Direct instruction interpretation
   - Simpler implementation
   - Easier debugging
   - Slower performance
   - Used for initial development and fallback

2. **JIT Mode**: Dynamic recompilation using Cranelift
   - Translates PowerPC → Cranelift IR → native code
   - Basic block detection and compilation
   - Code cache for compiled blocks
   - Much faster performance

#### Instruction Set

PowerPC has several instruction categories:
- Integer arithmetic and logical
- Floating-point operations
- Load/store with various addressing modes
- Branch and flow control
- System and privileged instructions
- AltiVec SIMD instructions (162 instructions)

### Memory System (newton-core/memory)

#### Address Space Layout

```
0x00000000 - 0x0FFFFFFF : RAM (up to 256MB)
0x10000000 - 0x7FFFFFFF : Extended RAM / Devices
0x80000000 - 0xEFFFFFFF : Memory-mapped I/O
0xF0000000 - 0xFEFFFFFF : Reserved
0xFF000000 - 0xFFEFFFFF : ROM (4MB)
0xFFF00000 - 0xFFFFFFFF : ROM (1MB, typical)
```

#### Memory Management

- **Physical Memory**: Direct RAM access
- **ROM**: Read-only, mapped at high addresses
- **MMIO**: Device registers mapped to specific addresses
- **Big-endian**: PowerPC native byte order
- **Address Translation**: BAT registers and page tables (TBD)

### Device Emulation (newton-devices)

#### Video Output
- Framebuffer device
- Multiple color depths (8/16/32-bit)
- Resolution configurable
- Memory-mapped for CPU access
- Rendered via wgpu

#### Input Devices

**Apple Desktop Bus (ADB)**:
- Classic Mac input system (Mac OS 8/9)
- Keyboard (address 2)
- Mouse (address 3)
- Polling-based protocol

**USB** (future):
- OHCI controller emulation
- For Mac OS X era

#### Storage
- IDE controller for hard disks
- SCSI for compatibility
- Disk image files

#### Audio
- Sound output device
- Sample-based playback

#### Network
- Ethernet controller
- TAP/TUN for network bridging

### User Interface (newton-ui)

The UI is split into two separate windows for optimal performance:

#### Emulation Display Window (wgpu)
- **Pure wgpu rendering** for maximum performance
- Direct framebuffer-to-screen rendering
- Full-screen capable
- Low latency input handling
- Dedicated window for the emulated Mac display
- No UI overlay on the actual emulation output

#### Debug/Config Window (egui + wgpu)
- **Separate egui window** for debugging and configuration
- Can be shown/hidden without affecting emulation
- Tools and panels:
  - Register inspector
  - Memory viewer (hex dump)
  - Disassembly view
  - Breakpoints/watchpoints
  - Step execution controls
  - Performance metrics
  - CPU settings
  - Memory size configuration
  - Device configuration
  - Display options

This separation ensures:
- Zero UI overhead on emulation rendering
- Debug tools don't impact emulation performance
- Clean separation of concerns
- Better multi-monitor support

## Data Flow

```
User Input → winit events
    ↓
Emulation Window (wgpu) ← Framebuffer ← Devices
    ↓ keyboard/mouse
    ↓
CPU ← fetch instruction ← Memory
    ↓ decode
    ↓ execute
    ↓ write back
Devices ← MMIO writes ← CPU
    ↓
Debug Window (egui) ← CPU/Memory state
    ↓ control commands
    └→ Emulator (pause/step/breakpoints)
```

## Performance Considerations

### JIT Compilation Strategy

1. **Block Detection**: Identify basic blocks (no branches within)
2. **IR Translation**: Convert PowerPC instructions to Cranelift IR
3. **Optimization**: Let Cranelift optimize the IR
4. **Code Generation**: Compile to native machine code
5. **Caching**: Store compiled blocks in hash map by address
6. **Linking**: Handle branches between blocks

### Memory Access Optimization

- Fast path for RAM access (bounds check + direct array access)
- MMIO device lookup only when needed
- Endianness handled via byteorder crate (may inline)

### Threading Model

Current design is single-threaded:
- CPU execution
- Device updates
- UI rendering

Future: Consider separating CPU execution to another thread with synchronization.

## Error Handling

Using Rust's Result type with custom Error enum:
- `Error::Cpu` - CPU execution errors
- `Error::Memory` - Invalid memory access
- `Error::Device` - Device operation failure
- `Error::Io` - File I/O errors

Errors are propagated up and logged, emulator can pause on error for debugging.

## Testing Strategy

### Unit Tests
- Individual instruction execution
- Memory operations
- Device functionality

### Integration Tests
- CPU + Memory interaction
- Full instruction sequences
- Device communication

### ROM Tests
- Execute actual ROM code
- Validate against known behavior

### OS Tests
- Boot Mac OS images
- Application execution

## Future Enhancements

1. **Save states**: Serialize entire emulator state
2. **Debugger enhancements**: Reverse debugging, time travel
3. **Performance profiling**: Built-in profiler for hot spots
4. **Multiple CPU cores**: Multi-processor emulation
5. **Hardware acceleration**: GPU-accelerated rendering effects
6. **Cross-platform**: Ensure works on Linux, macOS, Windows
