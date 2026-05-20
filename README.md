# NewtonEmu

A PowerPC Macintosh emulator written in Rust, targeting Mac OS 8 through Mac OS X 10.4 (Tiger).

## Features

- **PowerPC G4 (7400/7450) CPU emulation** with AltiVec SIMD support
- **OpenFirmware/Forth interpreter** - Complete CHRP boot script execution (19/19 NewWorld ROMs)
- **Separated frontend architecture**:
  - Pure wgpu display window for low-latency emulation rendering
  - Separate Qt 6 frontend for debugging, configuration, and tools
- **Hardware acceleration** with wgpu for graphics rendering
- **Memory-mapped RAM** for zero-overhead debugger access
- **Full peripheral support** (in development):
  - Framebuffer video output
  - Apple Desktop Bus (ADB) for classic Mac input
  - IDE/SCSI storage with ISO and raw disk support
  - Audio output
  - Networking

## Architecture

NewtonEmu uses a **separated architecture** with distinct CLI emulator and GUI frontend:

**Emulator Core (Rust)**:
- **newton-core**: Main emulator orchestration, OpenFirmware, ROM loading
- **newton-cpu**: PowerPC CPU implementation (interpreter and JIT compiler)
- **newton-devices**: Hardware peripheral emulation (ADB, storage, framebuffer)
- **newton-utils**: Shared utilities and error handling

**Frontend (C++23/Qt 6)** - See `frontend/`:
- Configuration editor (JSON-based)
- Live debugger (connects to emulator IPC)
- HFS+ file manager (offline tool)
- Network monitor (Wireshark-style)

The emulator runs as a CLI application and can operate headless for testing.

## Building

### Prerequisites

**For emulator:**
- Rust 2024 edition (install from [rustup.rs](https://rustup.rs))
- A GPU with Vulkan, Metal, or DirectX 12 support

**For Qt frontend (optional):**
- Qt 6.x
- C++23 compiler (GCC 11+, Clang 14+, or MSVC 2022+)
- CMake 3.20+

### Compile

**Emulator:**
```bash
cargo build --release
```

**Qt Frontend:**
```bash
cd frontend
./build.sh
# Or manually:
mkdir build && cd build
cmake ..
cmake --build .
```

## Running

**CLI emulator only:**
```bash
cargo run --release -- --rom roms/your-rom.rom --headless
```

**With display window:**
```bash
cargo run --release -- --rom roms/your-rom.rom
```

**With Qt frontend (recommended):**
```bash
./frontend/build/NewtonEmuFrontend
```

### Configuration

Configuration uses JSON format. Create a config file or use the Qt frontend's configuration editor.

Example `config.json`:
```json
{
  "cpu": {
    "model": "G4_7400",
    "clock_speed_mhz": 500
  },
  "memory": {
    "ram_size_mb": 256,
    "rom_path": "roms/your-rom.rom"
  },
  "display": {
    "width": 800,
    "height": 600,
    "color_depth": 32
  },
  "storage": [
    {
      "type": "cd",
      "path": "disks/MacOS9.iso"
    }
  ]
}
```

### ROM Files

NewtonEmu supports NewWorld CHRP boot ROMs. You must provide your own ROM dump from a real Macintosh.

**Tested ROMs:** All 19 NewWorld ROMs (1998-2003) are verified for boot script execution.

**ROM format:**
- NewWorld ROMs: 1-4MB, CHRP format with embedded Forth boot script
- Loaded at 0xFFC00000 (high memory)
- Entry point typically at offset +0x4100

Place your ROM file in the `roms/` directory.

## Usage

### CLI Options

```bash
newton-emu [OPTIONS]

Options:
  -r, --rom <FILE>           ROM file path
  -c, --config <FILE>        Configuration file (JSON)
      --ram <MB>             RAM size in MB (default: 256)
      --width <WIDTH>        Display width (default: 800)
      --height <HEIGHT>      Display height (default: 600)
      --cd <FILE>            Boot CD/DVD ISO
      --disk <FILE>          Boot disk image
  -p, --paused               Start paused
      --headless             No display window
      --debugger             Enable debugger IPC (stdin/stdout JSON)
      --gdb-server <ADDR>    Enable GDB server
  -h, --help                 Print help
  -V, --version              Print version
```

### Qt Frontend

The Qt frontend provides:
- **Configuration Editor**: Visual JSON config editing with storage device management
- **Debugger** (planned): Live CPU/memory inspection via IPC
- **HFS+ Manager** (planned): Offline disk image management
- **Network Monitor** (planned): Packet capture and analysis

Launch the emulator from the frontend's "Launch" button or run manually with `--debugger` for IPC integration.

## Development Status

NewtonEmu is in early development. Current status:

**Core:**
- [x] Project structure and build system
- [x] Memory-mapped RAM for debugger access
- [x] ROM loading (NewWorld CHRP ROMs)
- [x] OpenFirmware/Forth interpreter
- [x] Complete CHRP boot script execution (19/19 ROMs)
- [x] Qt 6 frontend with config editor
- [x] Debugger IPC protocol
- [x] Storage system (ISO, raw disk, SCSI)

**CPU:**
- [x] Basic CPU register definitions
- [x] Instruction decoder (100+ instructions)
- [x] Instruction interpreter (arithmetic, logical, shifts, compare, branches)
- [ ] CPU-Memory integration for load/store instructions
- [ ] Complete instruction set implementation
- [ ] JIT compiler with Cranelift
- [ ] Full AltiVec support

**Devices:**
- [x] Framebuffer device
- [x] ADB input devices (keyboard, mouse)
- [x] SCSI/IDE storage stubs
- [ ] Video output integration
- [ ] Audio output
- [ ] Networking

**Boot:**
- [ ] ROM entry point execution
- [ ] OpenFirmware client interface for ROM callbacks
- [ ] Device tree initialization from ROM
- [ ] Boot to Mac OS

## Roadmap

### Phase 1: OpenFirmware Integration (Current)
- Wire CPU to OpenFirmware client interface
- Execute ROM entry point
- Handle OF service calls from ROM code
- Complete device tree initialization

### Phase 2: CPU Core
- Complete PowerPC instruction set interpreter
- Integer, floating-point, and basic AltiVec instructions
- Memory access with proper endianness handling

### Phase 3: Graphics
- Full framebuffer implementation
- wgpu texture rendering
- Display output in window

### Phase 4: JIT Compiler
- Cranelift IR translation
- Basic block compilation
- Code cache management

### Phase 5: Peripherals
- Storage (IDE/SCSI) integration
- USB controller
- Audio
- Networking

### Phase 6: OS Boot
- Boot Mac OS 8/9
- Boot Mac OS X 10.0-10.4

## Testing

**ROM Boot Script Tests:**
```bash
# Verify all ROMs have valid CHRP boot scripts
cargo run --example scan_roms

# Test boot script execution
cargo run --example test_boot_script
```

**Storage Tests:**
```bash
# Test storage system
cargo test -p newton-devices --lib storage
```

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

NewtonEmu is licensed under the GNU General Public License v3.0. See [LICENSE](LICENSE) for details.

## References

- [PowerPC Programming Environments Manual](https://www.ibm.com/chips/techlib/techlib.nsf/products/PowerPC)
- [AltiVec Technology Programming Interface Manual](https://www.nxp.com/docs/en/reference-manual/ALTIVECPIM.pdf)
- [IEEE 1275-1994 OpenFirmware Standard](https://www.openfirmware.info/)
- Inside Macintosh series
- [QEMU PowerPC](https://www.qemu.org/) - Reference implementation

## Acknowledgments

- Inspired by QEMU, SheepShaver, and PearPC
- Built with Rust, wgpu, Qt 6, and Cranelift

---

**Note**: This is an emulator for educational purposes. You must own a legal copy of Mac OS and provide your own ROM files.
