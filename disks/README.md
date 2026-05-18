# NewtonEmu Disk Images and ISOs

Place your disk images and ISO files in this directory.

## Supported Formats

### CD/DVD Images
- `.iso` - ISO9660 CD/DVD images (read-only)
- Mac OS installer CDs
- Application/game CDs

### Hard Disk Images
- `.img` - Raw disk images (read/write)
- `.raw` - Raw disk images (read/write)

### Future Support
- `.qcow2` - QEMU copy-on-write format (snapshots, compression)
- `.vmdk` - VMware disk format
- `.vdi` - VirtualBox disk format

## Creating a Virtual Hard Disk

Use the emulator to create a new disk:

```bash
cargo run --example test_storage
```

Or manually create a raw disk image:

```bash
# Create a 1GB disk image
dd if=/dev/zero of=disks/harddisk.img bs=1M count=1024

# Or use fallocate (faster)
fallocate -l 1G disks/harddisk.img
```

## Mac OS Installation

1. Place a Mac OS installer ISO in this directory:
   - `disks/macos9_install.iso`
   - `disks/macosx_install.iso`

2. Create a virtual hard disk for installation:
   - Minimum 2GB for Mac OS 9
   - Minimum 4GB for Mac OS X

3. Run the emulator and boot from the CD

## Notes

- All files in this directory are ignored by git (except this README)
- You must provide your own Mac OS installation media
- ROM files go in the `roms/` directory
- ISO files must be ISO9660 format
- Disk images can be any size (limited by filesystem)
