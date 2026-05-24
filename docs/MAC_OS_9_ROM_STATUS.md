# Mac OS 9.2.2 Boot Status - ROM Requirements

## Current Status

NewtonEmu successfully boots through OpenFirmware and begins loading Mac OS 9.2.2, but fails with:

```
******************* MacOS: Fatal Error!  (0xF3B37FDB) *******************
Official Apple copyright message missing.
```

## Understanding the ROM Architecture

There are **TWO different ROM concepts** at play:

### 1. NewWorld Firmware ROM (✅ We Have This)
- **Location**: `roms/` directory (e.g., `2003-04-03 - Mac OS ROM 10.2.1.rom`)
- **Purpose**: Contains OpenFirmware and boot code
- **Loaded at**: 0xFFC00000 in emulator memory
- **Status**: Working correctly

### 2. Mac OS System ROM File (❌ We Don't Have This)
- **Location**: `System Folder:Mac OS ROM` on an **installed** Mac OS 9 system
- **Purpose**: Contains additional Mac OS system code and the copyright validation
- **Size**: ~3-4 MB
- **Status**: Missing - not present on the installer CD

## Why the Installer CD Doesn't Have It

The `macos-922-uni.iso` is an **installer CD**, not a bootable system:

```
Root directory:
├── Applications (Mac OS 9) [DIR]
├── Before You Install [FILE]
└── CD Extras [DIR]
```

**Missing**: `System Folder/` with the Mac OS ROM file

The Mac OS ROM file only exists on a **fully installed** system after running the installer.

## Boot Sequence Analysis

1. ✅ OpenFirmware starts from NewWorld ROM
2. ✅ Forth boot script executes
3. ✅ ELF bootloader extracted and loaded
4. ✅ Mac OS kernel begins initialization
5. ❌ Mac OS looks for copyright message in System ROM file
6. ❌ Validation fails → Fatal error 0xF3B37FDB

## Solutions

### Option 1: Pre-installed Disk Image (Recommended)
- Find or create a disk image with Mac OS 9.2.2 already installed
- Would have complete System Folder with Mac OS ROM file
- Can boot directly without installation

### Option 2: Run the Installer
- Implement enough emulator features to run the Mac OS installer
- Install Mac OS 9.2.2 to a blank disk image
- Then boot from the installed system
- **Requires**: More device emulation, possibly HFS+ write support

### Option 3: Patch the Copyright Check
- Locate the copyright validation code in ROM
- Patch it to skip the check
- **Risk**: May break other Mac OS functionality
- **Legal**: Modifying Apple ROM code

### Option 4: Extract from Real Mac
- Boot a real Mac or emulator with Mac OS 9 installed
- Copy the "Mac OS ROM" file from System Folder
- Load it into emulator at runtime
- **Legal**: Distribution of Apple ROM violates copyright

### Option 5: Alternative Boot Path
- Focus on OpenFirmware functionality first
- Test with alternative operating systems (Linux/BSD)
- Return to Mac OS 9 boot later with proper disk image

## Current Emulator Status

### ✅ Fully Working
- PowerPC CPU (G3/G4/G5 modes)
- MMU with BAT, TLB, and page tables
- All 48 load/store instructions with MMU translation
- OpenFirmware implementation
- Forth interpreter with boot script execution
- ELF loading
- LZSS decompression
- HFS filesystem reading
- SCSI/MESH storage controller
- Framebuffer and text console
- Basic video output

### 🔄 Partially Working
- Mac OS 9.2.2 boot (stops at copyright check)
- CD-ROM access (reads installer CD correctly)

### ❌ Not Yet Implemented
- HFS+ write support
- Full device tree
- Sound/audio
- Network
- Input devices (keyboard/mouse beyond basic)

## Recommendation

The cleanest path forward is **Option 1**: obtaining a pre-installed Mac OS 9.2.2 disk image or running the installer in another emulator first, then using that disk image with NewtonEmu.

This avoids:
- Legal issues with ROM distribution
- Complexity of patching Apple code
- Need for extensive installer support

## Next Steps

1. Attempt to find a pre-installed Mac OS 9.2.2 disk image
2. Test with alternative operating systems to verify hardware emulation
3. Implement remaining device emulation for installer support
4. Consider focusing on Mac OS X instead (different boot path)
