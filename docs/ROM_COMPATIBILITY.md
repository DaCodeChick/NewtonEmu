# Mac OS ROM Compatibility

NewtonEmu supports Mac OS ROMs from 1998 to 2003. This document describes ROM structure and compatibility.

## ROM Versions Tested

### Successfully Tested ROMs

All of the following ROMs have been verified to boot and execute the bootloader:

| ROM Version | Date       | Size  | ELF Offset | Entry Point | Notes |
|-------------|------------|-------|------------|-------------|-------|
| 1.1         | 1998-07-21 | 1.8MB | 0x3000     | 0x0020A1AC  | Early iMac |
| 1.1.2       | 1998-08-27 | 1.8MB | 0x3000     | (similar)   | |
| 1.2         | 1998-12-03 | 2.0MB | 0x4000     | 0x00208710  | |
| 1.2.1       | 1999-01-22 | 2.0MB | 0x4000     | 0x00208710  | |
| 1.4         | 1999-04-05 | 1.9MB | 0x4000     | 0x00208780  | |
| 1.6         | 1999-05-14 | 1.9MB | 0x4000     | 0x00208780  | |
| 2.5.1       | 1999-09-17 | 3.5MB | 0x4000     | 0x0020B468  | Reference ROM |
| 3.0         | 1999-09-27 | 2.2MB | 0x4000     | 0x0020B468  | |
| 1.8.1       | 1999-09-28 | 2.5MB | 0x4000     | 0x00208780  | |
| 3.7         | 2000-03-15 | 2.3MB | **0x5000** | 0x0020BD38  | New offset! |
| 3.8         | 2000-05-22 | 2.3MB | 0x5000     | (similar)   | |
| 6.1         | 2000-11-03 | 2.3MB | 0x5000     | (similar)   | |
| 7.5.1       | 2001-02-07 | 3.4MB | 0x5000     | (similar)   | |
| 8.4         | 2001-07-30 | 2.4MB | 0x5000     | (similar)   | |
| 9.0.1       | 2001-12-19 | 2.6MB | 0x5000     | (similar)   | |
| 9.1.1       | 2002-04-08 | 2.7MB | 0x5000     | (similar)   | |
| 9.6.1       | 2002-09-03 | 2.7MB | 0x5000     | (similar)   | |
| 9.8.1       | 2003-01-10 | 2.7MB | 0x5000     | (similar)   | |
| 10.2.1      | 2003-04-03 | 2.7MB | 0x5000     | 0x0020F078  | Latest |

## ROM Structure

All tested NewWorld ROMs follow this structure:

1. **CHRP Boot Script** (starts at 0x0000)
   - XML-like format starting with `<CHRP-BOOT>`
   - Ends with `</CHRP-BOOT>`
   - Length: ~12-14KB

2. **ELF Bootloader** (offset varies by ROM version)
   - **1998 early (1.1-1.1.2)**: 0x3000
   - **1998-1999 (1.2-3.0)**: 0x4000
   - **2000-2003 (3.7-10.2.1)**: 0x5000
   - PowerPC 32-bit big-endian executable
   - Type: EXEC (executable)
   - 3 program headers typically

3. **LZSS Compressed Data** (offset varies)
   - Identified by `prcl` magic number
   - Contains compressed ROM resources
   - Offset increases in later ROMs

4. **Native PowerPC Code and Data**
   - Mac OS ROM tables
   - OpenFirmware implementation
   - Device tree data
   - Boot resources

## ELF Offset Detection

The emulator automatically detects the ELF offset by searching for the ELF magic number (`0x7F454C46` / "\x7FELF") at these offsets:

```rust
let candidates = [0x3000, 0x4000, 0x5000, 0x8000];
```

If no ELF is found, the ROM may not be a NewWorld ROM or may be corrupted.

## Boot Process

1. **Reset**: CPU starts at ROM entry point
2. **Boot Script Execution**: Forth interpreter runs CHRP script
3. **ELF Loading**: Bootloader ELF is loaded into RAM at 0x00100000+
4. **CPU Jump**: PC set to ELF entry point
5. **Bootloader Runs**: Mac OS bootloader executes
6. **OpenFirmware Calls**: Bootloader queries device tree, writes debug output

## OpenFirmware Client Interface

All ROMs use the OpenFirmware client interface at `0x3000` (in RAM). Supported services:

- `finddevice` - locate device nodes
- `getprop` / `getproplen` - read properties
- `write` - console output (debug messages)
- `claim` / `release` - memory allocation
- Device tree traversal: `peer`, `child`, `parent`

## Debug Output

Most ROMs output debug information via OpenFirmware `write()` calls:

```
🖥️  AAPL,debug bit settings (-OR- bits together):
🖥️  0x1 = Print general informative messages.
🖥️  0x2 = Print formatted Mac OS tables (except config/universal info.
🖥️  0x4 = Print formatted config info table (formatted).
🖥️  0x8 = Dump Mac OS tables (except config/universal info).
🖥️  0x10 = Print node names while copying the device tree.
```

This confirms the bootloader is executing correctly.

## Known Issues

- Bootloaders currently loop in string formatting code after displaying debug info
- Full Mac OS boot requires more OpenFirmware services and device tree setup
- Some ROMs have slightly different output formatting

## Future Work

- Implement remaining OpenFirmware services
- Add proper device tree properties for real hardware
- Support reading LZSS compressed data
- Implement Mac OS ROM globals and descriptor tables
