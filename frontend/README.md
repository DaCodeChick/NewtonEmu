# NewtonEmu Frontend

Qt 6 GUI application for NewtonEmu PowerPC Macintosh Emulator.

## Features

### Implemented
- ✅ Configuration Editor
  - CPU & Memory settings
  - Display configuration
  - ROM file selection
  - Boot CD/DVD and disk selection
  - Network settings
  - Save/Load TOML configuration files
- ✅ Emulator Launcher
  - Launch emulator with current configuration
  - Process management
  - Output monitoring

### Coming Soon
- ⏳ Live Debugger (GDB protocol client)
- ⏳ HFS+ File Manager
- ⏳ Network Monitor
- ⏳ Disk Image Manager
- ⏳ Storage device list management

## Requirements

### Build Dependencies
- CMake 3.21 or newer
- Qt 6.5 or newer
- C++23 compatible compiler:
  - GCC 12+ or
  - Clang 16+ or
  - MSVC 2022+

### Runtime Dependencies
- NewtonEmu emulator binary (built from ../target/release/newton-emu)
- Qt 6 libraries

## Building

### Linux/macOS

```bash
# Install Qt 6 (Ubuntu/Debian)
sudo apt install qt6-base-dev qt6-tools-dev cmake build-essential

# Install Qt 6 (Fedora)
sudo dnf install qt6-qtbase-devel qt6-qttools-devel cmake gcc-c++

# Install Qt 6 (macOS with Homebrew)
brew install qt@6 cmake

# Build the frontend
cd frontend
mkdir build
cd build
cmake ..
make -j$(nproc)

# Run
./bin/NewtonEmuFrontend
```

### Windows

```powershell
# Install Qt 6 from https://www.qt.io/download
# Install CMake from https://cmake.org/download/
# Install Visual Studio 2022 with C++ support

# Build
cd frontend
mkdir build
cd build
cmake .. -G "Visual Studio 17 2022" -A x64
cmake --build . --config Release

# Run
.\bin\Release\NewtonEmuFrontend.exe
```

## Usage

1. **Configure the Emulator**
   - Set ROM file path
   - Configure CPU, memory, and display settings
   - Optionally add boot CD/DVD or disk images
   - Save configuration (File → Save)

2. **Launch Emulator**
   - Click "Launch Emulator" or press F5
   - The emulator will start with your configuration
   - Monitor status in the status bar

3. **Manage Configurations**
   - Create multiple configuration profiles
   - Save them with different names
   - Load previously saved configurations

## Configuration Files

Configuration files are TOML format, compatible with the emulator:

**Default Location:**
- Linux/macOS: `~/.config/newton-emu/config.toml`
- Windows: `%APPDATA%\newton-emu\config.toml`

**Example:**
```toml
[cpu]
model = "G4_7400"
clock_speed = 450

[memory]
ram_size_mb = 256
rom_path = "/path/to/rom.rom"

[display]
width = 800
height = 600
color_depth = 32

[storage]
boot_cd = "/path/to/macos9.iso"

[network]
enabled = false
```

## Architecture

The frontend communicates with the emulator through:

1. **Configuration Files**: TOML files passed via `--config` argument
2. **Process Management**: Launches and monitors the emulator process
3. **GDB Protocol** (future): For live debugging
4. **IPC Socket** (future): For HFS+ manager and network monitor

## Development

### Project Structure

```
frontend/
├── CMakeLists.txt          # Build configuration
├── src/
│   ├── main.cpp            # Application entry point
│   ├── mainwindow.{cpp,h,ui}    # Main window
│   └── config/
│       ├── configeditor.{cpp,h,ui}      # Configuration editor
│       ├── configmodel.{cpp,h}          # TOML config model
│       └── storagedevicewidget.{cpp,h,ui}  # Storage management
├── include/                # Public headers
├── resources/              # Icons, images, etc.
└── README.md
```

### Adding New Features

1. Create header in `include/`
2. Create implementation in `src/`
3. Add UI file in `src/` if using Qt Designer
4. Update `CMakeLists.txt` to include new files
5. Rebuild

### Code Style

- C++23 standard
- Follow Qt naming conventions
- Use signals/slots for communication
- Keep UI logic in widgets, business logic in models

## License

GNU General Public License v3.0

Copyright (C) 2026 NewtonEmu Contributors

## Links

- Main Project: https://github.com/YourUsername/NewtonEmu
- Qt Documentation: https://doc.qt.io/qt-6/
- NewtonEmu Docs: ../docs/FRONTEND_ARCHITECTURE.md
