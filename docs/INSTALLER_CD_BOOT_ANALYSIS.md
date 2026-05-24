# Mac OS 9 Installer CD Boot - Analysis

## The SheepShaver Approach

SheepShaver successfully installs Mac OS 9 from an installer CD with:
- Firmware ROM file
- Installer CD ISO
- Empty virtual disk
- No pre-installed system required

## Current Problem

NewtonEmu boots the firmware ROM, loads the Nanokernel ELF, and Mac OS begins initialization but fails with:
```
Fatal Error! (0xF3B37FDB)
Official Apple copyright message missing.
```

## Key Differences

### What SheepShaver Does
1. Loads firmware ROM
2. Provides CD and disk as SCSI devices
3. Does NOT automatically boot Mac OS from CD
4. User manually runs installer application
5. Installer creates System Folder on disk
6. Reboot from disk works normally

### What NewtonEmu Currently Does
1. Loads firmware ROM ✅
2. Provides CD and disk as SCSI devices ✅  
3. **Automatically executes ROM boot script** ← Issue
4. Boot script loads Nanokernel ELF
5. Mac OS tries to boot immediately
6. Mac OS looks for System ROM file → not found → fatal error

## The Root Cause

The firmware ROM's boot script **automatically attempts to boot Mac OS**. This works fine when:
- Booting from an installed system (has System Folder with Mac OS ROM file)
- NOT fine when booting from installer CD (no System Folder)

## Potential Solutions

### Option 1: Disable Auto-Boot
- Don't execute the ROM boot script automatically
- Drop to OpenFirmware prompt instead
- User can manually boot from CD or run installer
- **Requires**: OpenFirmware interactive prompt implementation

### Option 2: Check for Bootable System
- Before executing boot script, check if boot device has System Folder
- If not, don't try to boot Mac OS
- Show boot picker or OF prompt instead
- **Requires**: System Folder detection logic

### Option 3: Provide ROM in Memory
- The firmware ROM might already contain everything needed
- Mac OS check might be looking in wrong place
- Tell Mac OS the ROM is already loaded at a specific address
- **Requires**: Understanding Mac OS ROM memory layout

### Option 4: Patch the Copyright Check
- Find the check in the Nanokernel ELF
- Patch it to always succeed
- **Requires**: Disassembling and patching ELF code
- **Risk**: May break other functionality

### Option 5: Use Different ROM
- Try other ROM versions that might handle this differently
- Some ROMs might skip the check
- **Requires**: Testing all 19 ROM files we have

### Option 6: Boot from Disk Instead
- Configure to boot from the empty disk, not CD
- Disk has no bootloader → OpenFirmware falls back
- Might drop to OF prompt or show boot picker
- **Requires**: Changing boot device priority

## Investigation Needed

1. **Test if removing boot_cd helps** - Only configure boot_disk
2. **Test different ROM versions** - Try older/newer ROMs
3. **Examine error code 0xF3B37FDB** - What does it mean specifically?
4. **Study SheepShaver source** - How does it handle this?
5. **Check for blessed system folder** - Mac uses this to detect bootability

## Next Steps

1. Try booting with only boot_disk configured (no boot_cd)
2. See if that changes the boot behavior
3. Implement OpenFirmware prompt to allow manual device selection
4. Research Mac OS blessed system folder detection
