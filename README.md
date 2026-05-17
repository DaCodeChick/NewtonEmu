# NewtonEmu

A PowerPC Macintosh emulator written in Rust, targeting Mac OS 8 through Mac OS X 10.4 (Tiger).

## Features

- **PowerPC G4 (7400/7450) CPU emulation** with AltiVec SIMD support
- **Dynamic recompilation (JIT)** using Cranelift for high performance
- **Hardware acceleration** with wgpu for graphics rendering
- **Comprehensive debugging tools** with egui-based UI
- **Full peripheral support**:
  - Framebuffer video output
  - Apple Desktop Bus (ADB) for classic Mac input
  - USB controller
  - IDE/SCSI storage
  - Audio output
  - Networking

## Architecture

NewtonEmu is organized as a Rust workspace with multiple crates:

- **newton-core**: Main emulator orchestration, memory management, configuration
- **newton-cpu**: PowerPC CPU implementation (interpreter and JIT compiler)
- **newton-devices**: Hardware peripheral emulation
- **newton-ui**: User interface and debugging tools (egui + wgpu)
- **newton-utils**: Shared utilities and error handling

## Building

### Prerequisites

- Rust 2024 edition (install from [rustup.rs](https://rustup.rs))
- A GPU with Vulkan, Metal, or DirectX 12 support

### Compile

```bash
cargo build --release
```

## Running

```bash
cargo run --release
```

### Configuration

The emulator can be configured by editing the default configuration or loading a custom config file. Key settings include:

- CPU model (G3/G4/G5)
- RAM size
- ROM file path
- Display resolution and color depth
- JIT compilation enable/disable

### ROM Files

NewtonEmu requires a Mac ROM file to boot. You must provide your own ROM dump from a real Macintosh. Common ROM locations:

- Mac OS 8/9: Usually 4MB ROM at 0xFFC00000
- Early Mac OS X: 1MB ROM at 0xFFF00000

Place your ROM file in the `roms/` directory and configure the path in the emulator settings.

## Usage

### Controls

- **File → Load ROM**: Load a Mac ROM file
- **Emulation → Start/Stop**: Control emulation
- **Emulation → Step**: Execute a single instruction
- **View → Debugger**: Open the debugging interface

### Debugging

The built-in debugger provides:

- CPU register inspection (GPRs, FPRs, VRs)
- Memory viewer with hex dump
- Disassembly view
- Breakpoints and watchpoints
- Step-by-step execution

## Development Status

NewtonEmu is in early development. Current status:

- [x] Project structure and build system
- [x] Basic CPU register definitions
- [x] Instruction decoder (partial)
- [x] Instruction interpreter (basic operations)
- [x] Memory management system
- [x] ROM loading
- [x] Framebuffer device
- [x] ADB input devices
- [x] Basic UI framework
- [ ] Complete instruction set implementation
- [ ] JIT compiler
- [ ] Full AltiVec support
- [ ] Storage devices
- [ ] Audio output
- [ ] Networking
- [ ] Boot to Mac OS

## Roadmap

See [docs/architecture.md](docs/architecture.md) for detailed architecture documentation.

### Phase 1: CPU Core (Current)
- Complete PowerPC instruction set interpreter
- Integer, floating-point, and basic AltiVec instructions
- Memory access with proper endianness handling

### Phase 2: Boot to ROM
- Execute ROM code
- Basic device initialization
- Reach first instruction outside ROM

### Phase 3: Graphics
- Full framebuffer implementation
- wgpu texture rendering
- Display output in UI

### Phase 4: JIT Compiler
- Cranelift IR translation
- Basic block compilation
- Code cache management

### Phase 5: Peripherals
- Storage (IDE/SCSI)
- USB controller
- Audio
- Networking

### Phase 6: OS Boot
- Boot Mac OS 8/9
- Boot Mac OS X 10.0-10.4

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

NewtonEmu is licensed under the GNU General Public License v3.0. See [LICENSE](LICENSE) for details.

## References

- [PowerPC Programming Environments Manual](https://www.ibm.com/chips/techlib/techlib.nsf/products/PowerPC)
- [AltiVec Technology Programming Interface Manual](https://www.nxp.com/docs/en/reference-manual/ALTIVECPIM.pdf)
- Inside Macintosh series
- [QEMU PowerPC](https://www.qemu.org/) - Reference implementation

## Acknowledgments

- Inspired by QEMU, SheepShaver, and PearPC
- Built with amazing Rust crates: wgpu, egui, Cranelift

---

**Note**: This is an emulator for educational purposes. You must own a legal copy of Mac OS and provide your own ROM files.
