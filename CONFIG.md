# NewtonEmu Configuration

This is the unified configuration file for both the NewtonEmu backend (Rust) and frontend (Qt).

## Location

The default configuration file location is:
- **Linux/macOS**: `~/.config/newton-emu/config.json`
- **Windows**: `%APPDATA%\newton-emu\config.json`

You can also use this file as a template and specify a custom path:
```bash
# Backend
cargo run -- --config /path/to/config.json

# Frontend
# Loads from default location automatically
```

## Configuration Sections

### CPU
```json
"cpu": {
  "model": "G4_7400",      // G4_7400, G4_7450, G3, G5
  "clock_speed": 450       // MHz
}
```

### Memory
```json
"memory": {
  "ram_size_mb": 256,      // RAM in megabytes
  "rom_path": null         // Path to PowerPC ROM file (optional)
}
```

### Display
```json
"display": {
  "width": 800,            // Window width in pixels
  "height": 600,           // Window height in pixels
  "color_depth": 32        // Bits per pixel (8, 16, 24, 32)
}
```

### Storage
```json
"storage": {
  "boot_cd": null,         // Path to boot CD/DVD ISO
  "boot_disk": null,       // Path to boot disk image
  "scsi": [                // SCSI devices
    {
      "id": 0,             // SCSI ID (0-6, 7 is controller)
      "path": "disk.img",  // Path to disk image
      "readonly": false    // Read-only flag
    }
  ],
  "ide": [                 // IDE devices
    {
      "channel": 0,        // 0=primary, 1=secondary
      "device": 0,         // 0=master, 1=slave
      "path": "disk.img"   // Path to disk image
    }
  ]
}
```

Supported disk image formats:
- `.img`, `.raw` - Raw disk images
- `.iso` - CD/DVD ISO images (read-only)
- `.dmg` - Apple DMG images (read-only)
- `.toast` - Toast CD/DVD images (read-only)
- `.zip` - ZIP-compressed disk images

### Network
```json
"network": {
  "enabled": false,              // Enable network emulation
  "type": "slirp",               // Network type: slirp, tap, user
  "mac_address": "52:54:00:12:34:56"  // MAC address
}
```

### Debug
```json
"debug": {
  "gdb_server": null,            // GDB server address (e.g., "localhost:9000")
  "ipc_socket": null,            // IPC socket path for debugger
  "log_level": "info"            // trace, debug, info, warn, error
}
```

### Frontend (Qt GUI only)
```json
"frontend": {
  "emulator_path": null          // Path to newton-emu binary (auto-detected if null)
}
```

## Example Configurations

### Basic Mac OS 9 Boot
```json
{
  "cpu": {
    "model": "G4_7400",
    "clock_speed": 450
  },
  "memory": {
    "ram_size_mb": 256,
    "rom_path": "roms/PowerMacG4.rom"
  },
  "display": {
    "width": 800,
    "height": 600,
    "color_depth": 32
  },
  "storage": {
    "boot_disk": "disks/macos9.img",
    "scsi": []
  }
}
```

### Mac OS 9 CD Installation
```json
{
  "storage": {
    "boot_cd": "disks/macos-922-uni.iso",
    "boot_disk": "disks/new_disk.img",
    "scsi": []
  }
}
```

### Development with Debugging
```json
{
  "debug": {
    "gdb_server": "localhost:9000",
    "ipc_socket": "/tmp/newton-emu-debug.sock",
    "log_level": "debug"
  }
}
```

## Notes

- Both frontend and backend read and write the same config format
- The frontend section is ignored by the backend
- Null values mean the option is not set (uses defaults)
- Paths can be relative or absolute
- The config file is created automatically with defaults if it doesn't exist
