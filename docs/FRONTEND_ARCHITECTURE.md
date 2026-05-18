# Frontend Architecture - Qt 6 GUI

## Overview

NewtonEmu uses a **separation of concerns** architecture similar to SheepShaver and QEMU:

- **Emulator Core (Rust)**: CLI-only application that handles emulation and displays the framebuffer
- **Frontend GUI (C++23/Qt 6)**: Separate application providing configuration, debugging, and utility tools

This design provides:
- **Clean separation**: Emulator core has no GUI dependencies
- **Multiple frontends**: Different UIs can be built (Qt, web-based, mobile, etc.)
- **Better tooling**: Qt provides mature UI widgets and tools
- **Offline utilities**: Tools work even when emulator isn't running

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────┐
│                   Qt 6 Frontend (C++23)                  │
│                                                          │
│  ┌────────────────┐  ┌────────────────┐  ┌───────────┐ │
│  │ Configuration  │  │ Live Debugger  │  │ HFS+ Tool │ │
│  │    Editor      │  │                │  │           │ │
│  └────────────────┘  └────────────────┘  └───────────┘ │
│                                                          │
│  ┌────────────────┐  ┌────────────────┐                │
│  │ Network        │  │ Disk Image     │                │
│  │ Monitor        │  │ Manager        │                │
│  └────────────────┘  └────────────────┘                │
└─────────────────────────────────────────────────────────┘
                           │ IPC/Sockets
                           ▼
┌─────────────────────────────────────────────────────────┐
│            Emulator Core (Rust - CLI only)              │
│                                                          │
│  ┌──────────────────────────────────────────────────┐  │
│  │  GDB Server / Debug Protocol                     │  │
│  └──────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────┐  │
│  │  PowerPC CPU + Memory + Devices                  │  │
│  └──────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────┐  │
│  │  Display (wgpu framebuffer)                      │  │
│  └──────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

## Emulator Core (Rust)

### Design Principles

1. **CLI-only**: No integrated GUI code
2. **Pure wgpu**: Display uses only wgpu for framebuffer rendering
3. **Headless mode**: Can run without display for testing
4. **Debug protocol**: GDB-compatible remote debugging interface
5. **IPC interface**: Unix sockets or named pipes for external tools

### Command-Line Interface

```bash
# Basic usage
newton-emu --rom roms/rom.rom

# With storage
newton-emu --rom roms/rom.rom \
           --cd disks/macos9.iso \
           --disk disks/system.img

# Enable debug server
newton-emu --rom roms/rom.rom --debug-server localhost:9000

# Headless mode (no display)
newton-emu --rom roms/rom.rom --headless

# Configuration file
newton-emu --config ~/.config/newton-emu/config.toml
```

### Configuration File Format

**Location:** `~/.config/newton-emu/config.toml` (Linux/macOS) or `%APPDATA%\newton-emu\config.toml` (Windows)

```toml
[cpu]
model = "G4_7400"
clock_speed = 450  # MHz

[memory]
ram_size_mb = 256
rom_path = "/path/to/rom.rom"

[display]
width = 1024
height = 768
color_depth = 32

[storage]
# Boot from CD
boot_cd = "/path/to/macos9.iso"

# Hard disks (SCSI IDs)
[[storage.scsi]]
id = 0
path = "/path/to/system.img"

[[storage.scsi]]
id = 1
path = "/path/to/data.img"

# CD/DVD drives
[[storage.scsi]]
id = 3
path = "/path/to/software.iso"
readonly = true

# IDE devices
[[storage.ide]]
channel = 0  # primary
device = 0   # master
path = "/path/to/disk.img"

[network]
enabled = true
type = "slirp"  # User-mode networking
mac_address = "52:54:00:12:34:56"

[debug]
# Enable GDB server
gdb_server = "localhost:9000"

# Enable IPC for external tools
ipc_socket = "/tmp/newton-emu.sock"

# Logging level
log_level = "info"
```

### Debug Protocol

The emulator implements a GDB remote serial protocol server for debugging:

```
Protocol: GDB RSP (Remote Serial Protocol)
Transport: TCP socket
Commands:
  - Read/write registers
  - Read/write memory
  - Set breakpoints
  - Step/continue execution
  - Query state
```

### IPC Interface

For external tool communication (network monitor, HFS+ tool):

```
Protocol: JSON-RPC over Unix socket
Socket: /tmp/newton-emu-<pid>.sock

Methods:
  - emulator.pause()
  - emulator.resume()
  - emulator.reset()
  - emulator.get_state()
  - storage.list()
  - storage.read_block(device, lba, count)
  - storage.write_block(device, lba, data)
  - network.get_stats()
  - network.get_packets(count)
```

## Qt 6 Frontend (C++23)

### Components

#### 1. Configuration Editor

**Purpose:** Edit emulator settings before launch

**Features:**
- Visual ROM file selector
- Memory/CPU configuration
- Storage device management (add/remove/reorder)
- Network configuration
- Save/load configuration profiles
- Quick launch button

**UI Layout:**
```
┌─────────────────────────────────────────────────┐
│ NewtonEmu Configuration                          │
├─────────────────────────────────────────────────┤
│ [CPU & Memory]                                   │
│   CPU Model: [ G4 (7400) ▼ ]  Clock: [450] MHz │
│   RAM Size:  [256] MB                           │
│                                                  │
│ [ROM File]                                      │
│   Path: [/path/to/rom.rom      ] [Browse...]   │
│                                                  │
│ [Storage Devices]                               │
│   ┌───────────────────────────────────────────┐ │
│   │ SCSI ID 0: system.img (100 MB)     [Edit]│ │
│   │ SCSI ID 3: macos9.iso (CD-ROM)     [Edit]│ │
│   │                                           │ │
│   └───────────────────────────────────────────┘ │
│   [Add Device...] [Remove] [Move Up] [Move Dn]│
│                                                  │
│ [Network]                                       │
│   [ ] Enable networking                         │
│   Type: [ User-mode (SLIRP) ▼ ]               │
│                                                  │
│ [Profiles]                                      │
│   Current: [ Default ▼ ]  [Save] [Load] [Del] │
│                                                  │
│ [Launch Emulator] [Cancel]                      │
└─────────────────────────────────────────────────┘
```

#### 2. Live Debugger

**Purpose:** Debug running emulator (connects via GDB protocol)

**Features:**
- CPU register viewer (GPRs, SPRs, FPRs)
- Memory viewer/editor (hex + disassembly)
- Breakpoint management
- Step/continue/pause controls
- Call stack viewer
- Watch expressions
- Disassembly view with source mapping
- Log console

**UI Layout:**
```
┌─────────────────────────────────────────────────────────────┐
│ NewtonEmu Debugger - Connected to localhost:9000            │
├─────────────────────────────────────────────────────────────┤
│ [▶ Run] [⏸ Pause] [⏹ Stop] [⏭ Step] [⏩ Step Over] [↩ Step Out]│
├──────────────────────┬──────────────────────────────────────┤
│ Registers            │ Disassembly                          │
│ ┌──────────────────┐ │ ┌──────────────────────────────────┐ │
│ │ r0:  0x00000000  │ │ │ 0xFFC04100  mflr   r0           │ │
│ │ r1:  0x00EFFFF0  │ │ │ 0xFFC04104  stw    r0, 8(r1)    │ │
│ │ r2:  0xFFC08000  │ │ │ 0xFFC04108► bl     0xFFC04200   │ │
│ │ ...              │ │ │ 0xFFC0410C  lwz    r3, 0(r2)    │ │
│ └──────────────────┘ │ └──────────────────────────────────┘ │
│                      │                                       │
│ Breakpoints          │ Memory View                          │
│ ┌──────────────────┐ │ ┌──────────────────────────────────┐ │
│ │ ☑ 0xFFC04108     │ │ │ 0xFFC04100: 7C 08 02 A6 90 01 00│ │
│ │ ☑ 0xFFC05000     │ │ │ 0xFFC04108: 48 00 01 00 80 62 00│ │
│ └──────────────────┘ │ └──────────────────────────────────┘ │
├──────────────────────┴──────────────────────────────────────┤
│ Console                                                      │
│ ┌──────────────────────────────────────────────────────────┐ │
│ │ [INFO] Emulator started                                  │ │
│ │ [DEBUG] Breakpoint hit at 0xFFC04108                     │ │
│ └──────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

#### 3. HFS+ File Manager

**Purpose:** Browse and manage files on disk images (even when emulator is off)

**Features:**
- Browse HFS/HFS+ disk images
- Copy files to/from host filesystem
- Create/delete files and folders
- View file properties (type/creator codes, resource forks)
- Extract/inject resource forks
- Disk image creation wizard
- Format existing images

**UI Layout:**
```
┌─────────────────────────────────────────────────────────────┐
│ HFS+ File Manager - system.img                              │
├─────────────────────────────────────────────────────────────┤
│ File: [Open...] [Close] [Create New...] [Format...]        │
├──────────────────────┬──────────────────────────────────────┤
│ Folder Tree          │ File List                            │
│ ┌──────────────────┐ │ Name          Size    Type   Creator│
│ │ ▼ System Folder  │ │ ┌──────────────────────────────────┐ │
│ │   ▼ Preferences  │ │ │ Finder       245K   FNDR   MACS  │ │
│ │   ▼ Extensions   │ │ │ System       1.2M   zsys   MACS  │ │
│ │   ▶ Control      │ │ │ Mac OS ROM   2.0M   rom    AAPL  │ │
│ │     Panels       │ │ │ ...                              │ │
│ │ ▶ Applications   │ │ └──────────────────────────────────┘ │
│ │ ▶ Documents      │ │                                       │
│ └──────────────────┘ │ [Import] [Export] [Delete] [Info]   │
├──────────────────────┴──────────────────────────────────────┤
│ Properties                                                   │
│ Name: System        Type: zsys    Creator: MACS            │
│ Size: 1,234,567 bytes (Data: 1.1M, Resource: 100K)         │
│ Created: 1999-10-23 10:15:00   Modified: 1999-10-23 10:15:00│
│ [ ] Locked    [ ] Invisible    [ ] Bundle                  │
└─────────────────────────────────────────────────────────────┘
```

#### 4. Network Monitor

**Purpose:** Monitor emulator's network traffic (Wireshark-style)

**Features:**
- Live packet capture
- Protocol decoding (Ethernet, IP, TCP, UDP, AppleTalk, etc.)
- Packet filtering
- Search packets
- Export to PCAP format
- Statistics and graphs
- Connection tracking

**UI Layout:**
```
┌─────────────────────────────────────────────────────────────┐
│ Network Monitor - Connected to newton-emu                   │
├─────────────────────────────────────────────────────────────┤
│ [⏺ Capture] [⏹ Stop] [Clear] Filter: [tcp.port == 80     ]│
├─────────────────────────────────────────────────────────────┤
│ Packet List                                                 │
│ No.  Time      Source         Dest           Protocol  Info │
│ ┌──────────────────────────────────────────────────────────┐ │
│ │ 1   0.000000  52:54:00:12... 08:00:27:ab.. ARP     Who..│ │
│ │ 2   0.001245  192.168.1.10   192.168.1.1   TCP     [SYN]│ │
│ │ 3   0.001789  192.168.1.1    192.168.1.10  TCP     [ACK]│ │
│ │ ...                                                      │ │
│ └──────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│ Packet Details                                              │
│ ┌──────────────────────────────────────────────────────────┐ │
│ │ ▼ Ethernet II, Src: 52:54:00:12:34:56                   │ │
│ │   ▼ Internet Protocol Version 4, Src: 192.168.1.10      │ │
│ │     ▼ Transmission Control Protocol, Src Port: 54321    │ │
│ │       [SYN] Seq=0 Win=8192 Len=0                         │ │
│ └──────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│ Packet Bytes (Hex)                                          │
│ ┌──────────────────────────────────────────────────────────┐ │
│ │ 0000: 52 54 00 12 34 56 08 00 27 ab cd ef 08 00 45 00  │ │
│ │ 0010: 00 3c 1c 46 40 00 40 06 b1 e6 c0 a8 01 0a c0 a8  │ │
│ └──────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

#### 5. Disk Image Manager

**Purpose:** Create and manage virtual disk images

**Features:**
- Create new disk images (raw, QCOW2, etc.)
- Resize images
- Convert between formats
- Defragment images
- Compress/optimize images
- Image info viewer

### Technology Stack

**Language:** C++23
**Framework:** Qt 6.x
**Build System:** CMake
**Minimum Qt Version:** 6.5

**Required Qt Modules:**
- QtCore
- QtGui
- QtWidgets
- QtNetwork (for IPC/debug protocol)

**Optional Qt Modules:**
- QtCharts (for network statistics graphs)

### Project Structure

```
newton-frontend/
├── CMakeLists.txt
├── src/
│   ├── main.cpp
│   ├── mainwindow.{h,cpp}
│   ├── config/
│   │   ├── configeditor.{h,cpp}
│   │   ├── configmodel.{h,cpp}
│   │   └── storagedevicewidget.{h,cpp}
│   ├── debugger/
│   │   ├── debugwindow.{h,cpp}
│   │   ├── gdbclient.{h,cpp}
│   │   ├── registerwidget.{h,cpp}
│   │   ├── disassemblywidget.{h,cpp}
│   │   └── memorywidget.{h,cpp}
│   ├── hfstools/
│   │   ├── hfsmanager.{h,cpp}
│   │   ├── hfsfilesystem.{h,cpp}
│   │   └── resourcefork.{h,cpp}
│   ├── network/
│   │   ├── networkmonitor.{h,cpp}
│   │   ├── packetcapture.{h,cpp}
│   │   └── protocoldecoder.{h,cpp}
│   └── ipc/
│       ├── emulatorconnection.{h,cpp}
│       └── jsonrpc.{h,cpp}
├── resources/
│   ├── icons/
│   └── newton-frontend.qrc
└── README.md
```

## Communication Protocols

### 1. Configuration

**Method:** TOML configuration file  
**Location:** `~/.config/newton-emu/config.toml`  
**Flow:** Frontend writes → Emulator reads on launch

### 2. Live Debugging

**Protocol:** GDB Remote Serial Protocol (RSP)  
**Transport:** TCP socket  
**Port:** Configurable (default 9000)  
**Library:** Use existing GDB client libraries

### 3. Storage Access (HFS+ Tool)

**Protocol:** Custom JSON-RPC  
**Transport:** Unix socket / Named pipe  
**Methods:**
```json
// Request
{"jsonrpc": "2.0", "method": "storage.read_block", "params": {"device": 0, "lba": 100, "count": 1}, "id": 1}

// Response
{"jsonrpc": "2.0", "result": {"data": "base64..."}, "id": 1}
```

### 4. Network Monitoring

**Protocol:** Custom JSON-RPC  
**Transport:** Unix socket / Named pipe  
**Methods:**
```json
// Subscribe to packets
{"jsonrpc": "2.0", "method": "network.subscribe", "params": {}, "id": 1}

// Packet notification
{"jsonrpc": "2.0", "method": "network.packet", "params": {"timestamp": ..., "data": "base64...", "length": 64}}
```

**Alternative:** Export to PCAP file, frontend reads file

## Implementation Phases

### Phase 1: Core Simplification (Current Sprint)
- [ ] Remove egui dependencies from emulator
- [ ] Remove newton-ui crate
- [ ] Simplify main.rs to CLI-only with wgpu display
- [ ] Add --headless mode
- [ ] Update configuration to use TOML files
- [ ] Update documentation

### Phase 2: Debug Protocol (Week 1-2)
- [ ] Implement GDB RSP server in Rust emulator
- [ ] Add breakpoint support
- [ ] Add memory/register read/write
- [ ] Test with standard GDB client

### Phase 3: IPC Interface (Week 2-3)
- [ ] Design JSON-RPC API
- [ ] Implement Unix socket server in emulator
- [ ] Add storage access methods
- [ ] Add network packet streaming

### Phase 4: Qt Frontend - Config Editor (Week 3-4)
- [ ] Set up Qt project structure
- [ ] Implement configuration model
- [ ] Build configuration editor UI
- [ ] Add profile management
- [ ] Add emulator launcher

### Phase 5: Qt Frontend - Debugger (Week 5-6)
- [ ] Implement GDB client in Qt
- [ ] Build register viewer
- [ ] Build memory viewer
- [ ] Build disassembly view
- [ ] Add breakpoint UI

### Phase 6: Qt Frontend - HFS+ Tool (Week 7-8)
- [ ] Research HFS+ format
- [ ] Implement HFS+ parser (or use library)
- [ ] Build file browser UI
- [ ] Add import/export functionality
- [ ] Add resource fork editor

### Phase 7: Qt Frontend - Network Monitor (Week 9-10)
- [ ] Implement packet capture client
- [ ] Build packet list UI
- [ ] Add protocol decoder
- [ ] Add filtering
- [ ] Add PCAP export

## Benefits of This Architecture

### For Developers

✅ **Clean separation**: Emulator core is simpler, no GUI code  
✅ **Better testing**: Headless mode for CI/CD  
✅ **Multiple frontends**: Web UI, mobile app possible  
✅ **Language flexibility**: Use best tool for each job (Rust for emulation, C++/Qt for GUI)

### For Users

✅ **Familiar UI**: Qt provides native-looking widgets  
✅ **Offline tools**: HFS+ manager works without running emulator  
✅ **Better debugging**: Dedicated debugger window with professional features  
✅ **Network analysis**: Wireshark-style packet inspection

### For Project

✅ **Maintainability**: Smaller, focused components  
✅ **Testing**: Each component can be tested independently  
✅ **Performance**: No GUI overhead in emulator core  
✅ **Portability**: Emulator core easier to port to other platforms

## References

- **SheepShaver**: MacOS 9 emulator with separated GUI
- **QEMU**: Emulator core with multiple frontends (GTK, SDL, web)
- **GDB RSP**: https://sourceware.org/gdb/onlinedocs/gdb/Remote-Protocol.html
- **Qt Documentation**: https://doc.qt.io/qt-6/
- **HFS+ Format**: Apple Technical Note TN1150
- **JSON-RPC 2.0**: https://www.jsonrpc.org/specification
