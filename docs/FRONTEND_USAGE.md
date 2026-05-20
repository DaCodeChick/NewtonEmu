# Using the NewtonEmu Frontend

## Quick Start

The NewtonEmu frontend provides a GUI for configuring and launching the emulator.

### Running the Frontend

```bash
cd frontend/build/bin
./NewtonEmu
```

### Configuration

1. **CPU Settings**
   - Model: G3, G4 (7400/7450), or G5
   - Clock Speed: 400-1000 MHz

2. **Memory Settings**
   - RAM: 128-512 MB
   - ROM Path: Select a Mac OS NewWorld ROM file

3. **Display Settings**
   - Resolution: 640x480 to 1280x1024
   - Color Depth: 8, 16, or 32 bits

4. **Storage**
   - Boot CD: Mac OS 9 installation ISO
   - Boot Disk: Hard disk image
   - SCSI Devices: Additional SCSI devices (ID 0-7)

### Launching the Emulator

1. Configure your settings in the frontend
2. Click "Launch Emulator" or press F5
3. The emulator will start with a display window
4. The framebuffer output will show the emulated display

### Example Configuration

A test configuration has been created at:
```
~/.config/newton-emu/config.json
```

This configuration includes:
- PowerPC G4 7400 @ 450 MHz
- 256 MB RAM
- Mac OS ROM 1.4
- Mac OS 9.2.2 Universal Install CD

### Display Controls

The emulator uses wgpu for display rendering:
- Window size matches configured resolution
- Framebuffer updates are rendered in real-time
- Input handling for keyboard/mouse (coming soon)

### Current Status

✅ Working:
- Configuration editor
- JSON config loading/saving
- Emulator launching
- Storage device attachment (SCSI CD-ROM, hard disks)
- MESH SCSI controller MMIO
- Basic display framebuffer

🚧 In Progress:
- ROM boot sequence
- OpenFirmware client interface
- CD-ROM booting
- Input device handling

⏳ Coming Soon:
- Live debugger
- HFS+ file manager
- Network monitor
- Disk image creation tools

### Testing

To test the complete stack:

```bash
# Build the emulator (release mode for best performance)
cargo build --release

# Run the frontend
cd frontend/build/bin
./NewtonEmuFrontend
```

Or run directly with a config:

```bash
# Run with GUI display
./target/release/newton-emu --config ~/.config/newton-emu/config.json

# Run headless (for testing)
./target/release/newton-emu --config ~/.config/newton-emu/config.json --headless
```

### Command Line Options

```
newton-emu [OPTIONS]

Options:
  -r, --rom <FILE>           ROM file path
  -c, --config <FILE>        Configuration file (JSON)
      --ram <MB>             RAM size in MB [default: 256]
      --width <PIXELS>       Display width [default: 800]
      --height <PIXELS>      Display height [default: 600]
      --cd <FILE>            Boot CD/DVD ISO
      --disk <FILE>          Boot disk image
  -p, --paused               Start paused
      --headless             Headless mode (no display)
      --debugger             Enable debugger IPC
      --gdb-server <ADDR>    Enable GDB server
  -h, --help                 Print help
  -V, --version              Print version
```

### Troubleshooting

**Frontend doesn't start:**
- Ensure Qt 6 libraries are installed
- Check `ldd frontend/build/bin/NewtonEmu` for missing libraries

**Emulator doesn't launch:**
- Verify emulator binary exists: `target/release/newton-emu`
- Check ROM file path in config
- Ensure disk images exist and are readable

**Display issues:**
- Requires Vulkan or OpenGL support for wgpu
- Check `vulkaninfo` or `glxinfo` for GPU support

**Storage not working:**
- Verify ISO/disk images are readable
- Check SCSI ID conflicts (CD usually at ID 3, disks at 0-2)
- Supported formats: ISO, DMG, Toast, ZIP, raw disk images
