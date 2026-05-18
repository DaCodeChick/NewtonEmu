# NewtonEmu Debugger

A guest-system debugger for the emulated PowerPC Macintosh, inspired by Mesen2.

## Architecture

The debugger operates on the **emulated PowerPC system** (guest), not the Rust emulator itself (host).

```
┌─────────────────────┐          ┌──────────────────────┐
│   Qt Frontend       │          │   Rust Emulator      │
│                     │          │                      │
│  ┌──────────────┐   │          │  ┌───────────────┐   │
│  │   Debugger   │   │  stdin/  │  │  Debugger IPC │   │
│  │     UI       │───┼─stdout───┼─▶│    Handler    │   │
│  └──────────────┘   │  JSON    │  └───────────────┘   │
│         │           │          │         │            │
│         │           │          │         ▼            │
│         │           │          │  ┌───────────────┐   │
│         │           │   mmap   │  │   Debugger    │   │
│         └───────────┼──────────┼─▶│     Core      │   │
│                     │          │  └───────────────┘   │
│                     │          │         │            │
│                     │          │         ▼            │
│                     │          │  ┌───────────────┐   │
│                     │   mmap   │  │  Memory-Mapped│   │
│                     │──────────┼─▶│      RAM      │   │
│                     │          │  └───────────────┘   │
└─────────────────────┘          └──────────────────────┘
                                           │
                                           ▼
                                 /tmp/newton_emu_ram_<pid>.bin
```

## Features

### Execution Control
- **Pause/Resume**: Stop and continue emulation
- **Step Into**: Execute one instruction
- **Step Over**: Execute but skip over function calls
- **Step Out**: Run until current function returns

### Breakpoints
- **Execution breakpoints**: Break when PC reaches an address
- **Memory read breakpoints**: Break when memory is read from an address
- **Memory write breakpoints**: Break when memory is written to an address
- **Memory access breakpoints**: Break on both read and write

### State Inspection
- **CPU registers**: All GPRs, FPRs, SPRs (PC, LR, CTR, CR, XER)
- **Memory**: Direct zero-copy access via memory-mapped file
- **Call stack**: Function call tracking
- **Instruction count**: Total instructions executed

### Memory Access
RAM is **always** memory-mapped to `/tmp/newton_emu_ram_<pid>.bin` for:
- Zero serialization overhead
- No IPC latency
- Real-time memory inspection
- Direct memory modification (use with caution!)

## Command-Line Usage

### Start with Debugger
```bash
# Windowed mode with debugger
newton-emu --debugger --rom path/to/rom.ndrv

# Headless mode with debugger
newton-emu --headless --debugger --rom path/to/rom.ndrv
```

The emulator will:
1. Create memory-mapped RAM file
2. Start debugger IPC handler (stdin/stdout)
3. Log RAM path: `RAM mapped to file: /tmp/newton_emu_ram_12345.bin`

### Example Session
```bash
# Start emulator
newton-emu --headless --debugger --rom rom.ndrv &

# Send commands via stdin
echo '{"command":"pause"}' | nc localhost -
echo '{"command":"get_cpu_state"}' | nc localhost -
echo '{"command":"add_breakpoint","address":4096,"type":"execute"}' | nc localhost -
echo '{"command":"resume"}' | nc localhost -
```

## IPC Protocol

### Communication
- **Transport**: stdin (commands) / stdout (responses)
- **Format**: JSON, one message per line
- **Threading**: Background thread reads commands without blocking emulation

### Commands

#### Enable/Disable Debugger
```json
{"command":"enable"}
{"command":"disable"}
```

#### Execution Control
```json
{"command":"pause"}
{"command":"resume"}
{"command":"step_into"}
{"command":"step_over"}
{"command":"step_out"}
```

#### Breakpoints
```json
{"command":"add_breakpoint","address":4096,"type":"execute"}
{"command":"add_breakpoint","address":8192,"type":"memory_read"}
{"command":"remove_breakpoint","address":4096,"type":"execute"}
{"command":"get_breakpoints"}
```

Breakpoint types: `execute`, `memory_read`, `memory_write`, `memory_access`

#### State Inspection
```json
{"command":"get_cpu_state"}
```

Response:
```json
{
  "response":"cpu_state",
  "state":{
    "pc":4096,
    "lr":8192,
    "ctr":0,
    "cr":0,
    "xer":0,
    "gpr":[0,0,0,...],
    "fpr":[0.0,0.0,0.0,...],
    "execution_state":{"state":"paused"},
    "instruction_count":12345
  }
}
```

#### Memory Access
```json
{"command":"get_memory","address":4096,"length":256}
```

Response (redirects to mmap):
```json
{
  "response":"error",
  "message":"Use memory-mapped file instead: /tmp/newton_emu_ram_12345.bin"
}
```

**Preferred method**: Frontend should `mmap` the file directly.

### Responses

#### Success
```json
{"response":"ok"}
```

#### Error
```json
{"response":"error","message":"Debugger not enabled"}
```

#### CPU State
See above example.

#### Breakpoints
```json
{
  "response":"breakpoints",
  "breakpoints":[
    {"address":4096,"type":"execute","enabled":true,"hit_count":5},
    {"address":8192,"type":"memory_read","enabled":true,"hit_count":0}
  ]
}
```

## Memory-Mapped RAM Access

### C++ / Qt Example
```cpp
#include <QFile>
#include <sys/mman.h>
#include <fcntl.h>

// Open mmap file
QString ramPath = "/tmp/newton_emu_ram_12345.bin";
int fd = open(ramPath.toUtf8().constData(), O_RDWR);
size_t ramSize = 256 * 1024 * 1024; // 256 MB

// Map into memory
void* ramPtr = mmap(nullptr, ramSize, PROT_READ | PROT_WRITE, 
                    MAP_SHARED, fd, 0);

// Read guest memory at address 0x1000
uint32_t* guestMem = (uint32_t*)ramPtr;
uint32_t value = be32toh(guestMem[0x1000 / 4]);

// Write guest memory (use with caution!)
guestMem[0x1000 / 4] = htobe32(0x12345678);

// Cleanup
munmap(ramPtr, ramSize);
close(fd);
```

### Python Example
```python
import mmap
import struct

# Open mmap file
with open('/tmp/newton_emu_ram_12345.bin', 'r+b') as f:
    mm = mmap.mmap(f.fileno(), 256 * 1024 * 1024)
    
    # Read 32-bit big-endian value at address 0x1000
    value = struct.unpack('>I', mm[0x1000:0x1004])[0]
    print(f"Value at 0x1000: 0x{value:08x}")
    
    # Write value (use with caution!)
    mm[0x1000:0x1004] = struct.pack('>I', 0x12345678)
    
    mm.close()
```

## Implementation Details

### Components

1. **`debugger.rs`**: Core debugger state and logic
   - Breakpoint management
   - Execution state tracking
   - Call stack management

2. **`debugger_protocol.rs`**: JSON protocol DTOs
   - Command/response types
   - Serialization/deserialization

3. **`debugger_ipc.rs`**: IPC handler
   - Stdin reader thread
   - Command dispatcher
   - Response writer

4. **`memory.rs`**: Memory-mapped RAM
   - Always uses `memmap2`
   - Creates `/tmp/newton_emu_ram_<pid>.bin`
   - Cleaned up on exit

### Performance

- **IPC**: Background thread, non-blocking
- **Memory access**: Zero-copy via mmap
- **Overhead**: Negligible when debugger disabled
- **Breakpoints**: Checked before each instruction (minimal overhead)

## Qt Frontend Integration

See `frontend/` directory for Qt 6 C++ implementation.

### Required Qt Components
- `QProcess` for emulator launching
- `QJsonDocument` for JSON protocol
- Platform mmap APIs for memory access
- `QHexEdit` or custom widget for memory viewer
- `QTreeView` for register display

### Example Workflow
1. Frontend launches emulator with `--debugger`
2. Frontend captures stdout, parses RAM path
3. Frontend mmaps RAM file
4. Frontend sends JSON commands via stdin
5. Frontend receives JSON responses via stdout
6. Frontend displays memory/registers in real-time

## Testing

Run the test script:
```bash
./test_debugger.sh
```

Manual testing:
```bash
# Terminal 1: Start emulator
cargo run --release -- --headless --debugger --rom rom.ndrv

# Terminal 2: Send commands
echo '{"command":"enable"}' | nc localhost -
echo '{"command":"get_cpu_state"}' | nc localhost -
```

## Limitations

- **Disassembly**: Not yet implemented (TODO: add PowerPC disassembler)
- **Symbols**: No symbol file support (future enhancement)
- **Watchpoints**: Limited to specific addresses (no range support yet)
- **Thread safety**: Single-threaded emulator only (multi-threaded mode not supported)

## Future Enhancements

- PowerPC disassembler integration (capstone or custom)
- Symbol file support (DWARF, map files)
- Memory range breakpoints
- Conditional breakpoints with expressions
- Execution trace recording
- Time-travel debugging (save/restore state)
- Performance profiling
