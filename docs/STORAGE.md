# Storage Device Implementation

This document describes the virtual storage device architecture implemented for NewtonEmu.

## Overview

The storage system provides emulation for:
- **CD/DVD drives** - ISO9660 image files via SCSI
- **Hard disks** - Raw disk images via SCSI or IDE/ATA
- **Apple Disk Images** - DMG format support with automatic extraction
- **Roxio Toast Images** - Toast CD/DVD image format support
- **ZIP archives** - Automatic extraction and format detection for compressed disk images
- **Future**: SCSI tape drives, floppy disk drives

## Architecture

### Block Device Abstraction

All storage devices implement the `BlockDevice` trait:

```rust
pub trait BlockDevice: Send + Sync {
    fn info(&self) -> &DeviceInfo;
    fn read_blocks(&self, lba: u64, count: u32, buffer: &mut [u8]) -> Result<()>;
    fn write_blocks(&mut self, lba: u64, count: u32, buffer: &[u8]) -> Result<()>;
    fn flush(&mut self) -> Result<()>;
    fn media_present(&self) -> bool;
}
```

This abstraction allows different storage backends (ISO, raw disk, DMG, Toast, ZIP) to be used interchangeably.

### Device Types

#### IsoImage
- Read-only CD/DVD image support
- Reads ISO9660 formatted disc images
- Reports as CD-ROM device type
- Supports sector size of 2048 bytes (standard for CDs)

#### RawDiskImage
- Read/write hard disk image support
- Flat file format (no compression or snapshots)
- Supports arbitrary sizes
- Block size of 512 bytes (standard for HDDs)
- Can create new images or open existing ones

#### DmgImage
- Apple Disk Image (.dmg) support
- Read-only access to DMG files
- Uses the `udif` crate for UDIF format parsing
- Automatically extracts the main partition to `/tmp/newton_dmg_<pid>.raw`
- Wraps the extracted data as a `RawDiskImage`
- Supports compressed and uncompressed DMG formats

#### ToastImage
- Roxio Toast CD/DVD image (.toast) support
- Read-only access to Toast format files
- Supports Toast v1, v2, and v3 formats
- Auto-detects if the Toast file is a simple ISO wrapper or requires extraction
- For ISO-wrapped files, uses `IsoImage` directly
- For other formats, extracts to `/tmp/newton_toast_<pid>.raw`

#### ZipImage
- ZIP archive (.zip) disk image support
- Automatically extracts the first disk image found in the archive
- Detects format by extension: .iso, .img, .dmg, .toast, .bin, .cdr
- Falls back to auto-detection by examining file contents
- Wraps the extracted image in the appropriate handler (IsoImage, DmgImage, ToastImage, or RawDiskImage)
- Progress logging for large archives (logs every 100MB)
- Read-only access

### SCSI Emulation

The `ScsiDevice` wrapper provides SCSI command emulation on top of any `BlockDevice`:

**Supported Commands:**
- `TEST UNIT READY` (0x00) - Check if device is ready
- `REQUEST SENSE` (0x03) - Get error information
- `INQUIRY` (0x12) - Get device identification
- `READ CAPACITY` (0x25) - Get device size
- `READ(10)` (0x28) - Read blocks
- `WRITE(10)` (0x2A) - Write blocks
- `READ TOC` (0x43) - Get CD table of contents

**SCSI Sense Data:**
The implementation maintains proper sense keys for error reporting:
- `NoSense` - No error
- `NotReady` - Device not ready (e.g., no media)
- `IllegalRequest` - Invalid command or parameter
- `HardwareError` - Device I/O failure

### Storage Bus

The `StorageBus` manages all storage devices:

**SCSI Bus:**
- Up to 8 devices (IDs 0-7)
- Typical configuration:
  - ID 0-2: Hard disks
  - ID 3-6: CD/DVD drives
  - ID 7: Usually reserved for host adapter

**IDE Bus:**
- Two channels (primary/secondary)
- Two devices per channel (master/slave)
- Typical configuration:
  - Primary master (0,0): Main hard disk
  - Primary slave (0,1): Second hard disk
  - Secondary master (1,0): CD/DVD drive
  - Secondary slave (1,1): Optional second CD/DVD

## Usage Examples

### Creating a Virtual Hard Disk

```rust
use newton_devices::storage::{RawDiskImage, StorageBus};
use std::sync::{Arc, RwLock};

// Create a 1GB disk image
let disk = RawDiskImage::create("disk.img", 1024)?;
let disk: Arc<RwLock<dyn BlockDevice>> = Arc::new(RwLock::new(disk));

// Attach to SCSI bus
let mut bus = StorageBus::new();
bus.attach_scsi(0, disk)?;
```

### Mounting a CD-ROM ISO

```rust
use newton_devices::storage::{IsoImage, StorageBus};
use std::sync::{Arc, RwLock};

// Open an ISO image
let iso = IsoImage::open("macos9.iso")?;
let iso: Arc<RwLock<dyn BlockDevice>> = Arc::new(RwLock::new(iso));

// Attach to SCSI bus as CD-ROM
let mut bus = StorageBus::new();
bus.attach_scsi(3, iso)?;
```

### Opening an Apple Disk Image (DMG)

```rust
use newton_devices::storage::{DmgImage, StorageBus};
use std::sync::{Arc, RwLock};

// Open a DMG file (automatically extracts to temp file)
let dmg = DmgImage::open("MacOS9_Install.dmg")?;
let dmg: Arc<RwLock<dyn BlockDevice>> = Arc::new(RwLock::new(dmg));

// Attach to SCSI bus
let mut bus = StorageBus::new();
bus.attach_scsi(3, dmg)?;
```

### Opening a Toast Image

```rust
use newton_devices::storage::{ToastImage, StorageBus};
use std::sync::{Arc, RwLock};

// Open a Toast CD/DVD image
let toast = ToastImage::open("disc.toast")?;
let toast: Arc<RwLock<dyn BlockDevice>> = Arc::new(RwLock::new(toast));

// Attach to SCSI bus
let mut bus = StorageBus::new();
bus.attach_scsi(3, toast)?;
```

### Opening a ZIP Archive with Disk Image

```rust
use newton_devices::storage::{ZipImage, StorageBus};
use std::sync::{Arc, RwLock};

// Open a ZIP archive (automatically extracts and detects format)
let zip = ZipImage::open("MacOS9_Install.zip")?;
let zip: Arc<RwLock<dyn BlockDevice>> = Arc::new(RwLock::new(zip));

// Attach to SCSI bus
let mut bus = StorageBus::new();
bus.attach_scsi(3, zip)?;
```

### Reading/Writing Data

```rust
// Write a block
let data = vec![0x42u8; 512];
disk.write_blocks(0, 1, &data)?;

// Read it back
let mut buffer = vec![0u8; 512];
disk.read_blocks(0, 1, &mut buffer)?;
```

### Executing SCSI Commands

```rust
// Get device from bus
let scsi_dev = bus.scsi_device_mut(0).unwrap();

// Execute INQUIRY command
let cdb = [0x12, 0, 0, 0, 36, 0];
let mut data = vec![0u8; 36];
let (status, len) = scsi_dev.execute_command(&cdb, &mut data)?;

// Parse response
let vendor = std::str::from_utf8(&data[8..16])?;
let product = std::str::from_utf8(&data[16..32])?;
```

## Testing

Run the storage test suite:

```bash
cargo run --example test_storage
```

This test:
1. Creates a 100MB virtual disk
2. Writes and reads test data
3. Attaches the disk to a SCSI bus
4. Optionally mounts ISO images if available
5. Executes various SCSI commands
6. Verifies all operations complete successfully

## Integration with Emulator

### OpenFirmware Device Tree

Storage devices should be exposed in the device tree:

```
/pci@f2000000
  /mac-io@17
    /ata-3@20000
      /disk@0           (IDE primary master)
    /ata-4@21000
      /disk@0           (IDE secondary master)
      /cdrom@1          (IDE secondary slave)
  /scsi@18
    /disk@0             (SCSI ID 0)
    /disk@1             (SCSI ID 1)
    /cdrom@3            (SCSI ID 3)
```

### Memory-Mapped I/O

Storage controllers need MMIO regions:
- **IDE**: Typically at 0x80013000 (cmd) + 0x80013160 (ctrl)
- **SCSI**: Typically at 0x80010000

### Interrupts

Storage devices should generate interrupts on:
- Command completion
- DMA transfer completion
- Error conditions

## File Formats

### Raw Disk Images
- Simple flat file format
- Size = number of blocks × block size
- No compression or metadata
- Compatible with QEMU, VirtualBox, VMware raw format

### ISO9660 Images
- Standard CD/DVD image format
- Read-only
- Sector size: 2048 bytes
- Compatible with all major CD burning/ISO tools

## Future Enhancements

### Short-term
- [ ] Wire storage into the main emulator
- [ ] Add device tree nodes for storage devices
- [ ] Implement IDE/ATA controller MMIO
- [ ] Implement SCSI controller (e.g., Symbios 53C94)
- [ ] Test with Mac OS installer ISOs
- [ ] Add OpenFirmware `block-name` and `block-size` properties

### Medium-term
- [ ] DMA support for faster transfers
- [ ] ATAPI (IDE CD-ROM) support
- [ ] Multiple LUN support
- [ ] HFS/HFS+ awareness for better integration
- [ ] Disk image format auto-detection

### Long-term
- [ ] QCOW2 format support (snapshots, compression)
- [ ] VMDK format support
- [ ] VDI format support
- [ ] Network block device (NBD) support
- [ ] Floppy disk controller
- [ ] SCSI tape drive emulation

## Performance Considerations

- Block reads/writes go through `RwLock` for thread safety
- ISO images use `BufReader` for efficient reading
- Raw disks use `BufReader`/`BufWriter` for buffered I/O
- Consider implementing a block cache layer for frequently accessed blocks
- DMA would bypass CPU for large transfers

## Compatibility

### Tested ROM Versions
- The storage system is designed to work with all NewWorld ROMs (1998-2003)
- Bootloader expects storage devices on SCSI bus (especially CD-ROM)
- Hard disks can be on either SCSI or IDE

### Mac OS Versions
- Mac OS 8.6 and later support NewWorld machines
- Mac OS 9.0-9.2.2 fully supported
- Mac OS X 10.0-10.4 should work with appropriate ROM

## Debugging

Enable storage debug logging:

```bash
RUST_LOG=newton_devices::storage=debug cargo run
```

This will show:
- Device attachment/detachment
- SCSI command execution
- Block reads/writes
- Error conditions

## Implementation Status

✅ **Completed:**
- Block device abstraction
- ISO image support
- Raw disk image support
- SCSI command emulation
- Storage bus management
- Comprehensive testing

🚧 **In Progress:**
- Integration with emulator
- Device tree registration
- Controller MMIO implementation

⏳ **Planned:**
- IDE/ATA controller
- Advanced disk formats
- Performance optimizations

## ROM Boot Script Compatibility

The Forth/OpenFirmware interpreter successfully executes CHRP boot scripts from all tested NewWorld ROMs:

- ✅ 19/19 ROMs (July 1998 - April 2003) execute completely
- Spans Mac OS ROM versions 1.1 through 10.2.1
- Covers iMac, Power Mac G3/G4/G5, PowerBook, and iBook models

All ROMs follow the same boot script pattern:
1. Setup device tree nodes (`/openprom`, `/chosen`)
2. Define memory management helpers
3. Allocate memory regions via OpenFirmware claim methods
4. Copy ELF and compressed ROM images to memory
5. Create `/rom/macos` node with toolbox-image property
6. Transfer control via `init-program` and `go`

The interpreter handles all required Forth constructs:
- String literals in colon definitions (`" claim"`)
- Print strings (`."`), comments (`\`), abort strings (`abort"`)
- Control flow (`if/then/else`, `begin/until`, `do/loop`)
- Values and constants
- Exception handling (`catch`, `throw`)
- OpenFirmware-specific words (`$call-method`, `get-package-property`, etc.)
