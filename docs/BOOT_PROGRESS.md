# NewWorld ROM Boot Progress

## Summary

We've made significant progress on booting Mac OS 9 from NewWorld ROMs. The emulator now correctly executes PowerPC ROM code and processes OpenFirmware calls.

## Changes Made

### 1. Implemented 68k CPU Emulator (`crates/newton-m68k/`)
- Full Motorola 68000 CPU core with all registers
- 12 addressing modes
- Core instruction set (data movement, arithmetic, logical, branches, system)
- Working test example demonstrating MOVEQ and ADDQ
- See `docs/M68K_EMULATOR.md` for full documentation

### 2. Fixed NewWorld ROM Loading (`crates/newton-core/src/rom.rs`)
- **Previously**: Decompressed LZSS/parcels data and treated it as the ROM
- **Now**: Keep raw NewWorld ROM file intact
- **Entry Point**: Set to ELF offset (0x4100) for PowerPC boot code
- **Memory Map**: ROM loaded at `0xFFFFFFFF - rom_size + 1`

### 3. Boot Flow Understanding

**NewWorld ROM Structure:**
```
Offset 0x0000: <CHRP-BOOT> XML script
Offset 0x4000: ELF header (magic: 0x7F454C46)
Offset 0x4100: PowerPC boot code (mflr r0, stmw, etc.)
Offset 0x15690: LZSS-compressed 68k ROM data (2.1MB compressed)
```

**Correct Boot Sequence:**
1. PowerPC CPU starts at ROM base + 0x4100 (ELF code)
2. PPC code parses CHRP boot script
3. PPC code decompresses LZSS data to 0xFFC00000 (for 68k Toolbox)
4. PPC code initializes OpenFirmware device tree
5. PPC code loads Mac OS and sets up mixed-mode environment
6. Mac OS uses both PPC (system) and 68k (Toolbox) code

## Current Status

### ✅ Working
- PowerPC ROM code execution from ELF section
- OpenFirmware client interface calls (write, finddevice, getprop, claim, etc.)
- Memory mapping and MMU
- Execution trace buffer for debugging
- Device tree setup

### ⚠️ Partially Working
- Boot progresses to Mac OS initialization
- OpenFirmware text output displays correctly
- ROM reaches parcel/copyright check

### ❌ Not Working Yet
- **Current Blocker**: "Official Apple copyright message missing" error
- **Cause**: ROM code expects decompressed 68k ROM at 0xFFC00000
- **Solution Needed**: ROM boot script should decompress LZSS data, but it's not happening

## Boot Log Analysis

### Successful Execution
```
PC=0x0020A56C  inst=0x83A2FF64  (lwz r29, -156(r2))
PC=0x0020A570  inst=0x7C9F2378  (mr r31, r4)
...
PC=0x0020C49C  inst=0x7C0903A6  (mtctr r0)
PC=0x0020C4A0  inst=0x804C0004  (lwz r2, 4(r12))
PC=0x0020C4A4  inst=0x4E800420  (bctr)
```

### Fatal Error
```
🖥️  ******************* MacOS: Fatal Error!  (0xF3B37FDB) *******************
🖥️  Official Apple copyright message missing.
```

Then jumps to `PC=0x00000000` (null function pointer).

## Next Steps

### High Priority
1. **Investigate ROM Decompression**
   - The boot script should decompress LZSS data to 0xFFC00000
   - Check why the decompression isn't happening
   - May need to implement LZSS decompression in PPC code path

2. **Device Tree Setup**
   - Ensure `/rom/macos` node exists with proper properties
   - Set `AAPL,toolbox-image,lzss` or `AAPL,toolbox-parcels` property
   - Verify ROM base/size are correctly advertised

3. **Debug Null Function Call**
   - Trace why code jumps to 0x00000000
   - May be missing exception handler or callback
   - Could be uninitialized function pointer table

### Medium Priority
4. **Integrate 68k Emulator**
   - Add `M68k` instance to main emulator
   - Implement memory interface bridge
   - Create shared memory space for both CPUs

5. **Implement Mixed Mode Manager**
   - Detect 68k vs PPC code addresses
   - Handle transitions between CPU modes
   - Save/restore CPU state on switches

6. **RTAS Support**
   - ROM code looks for `/rtas` device
   - Implement basic RTAS services
   - May be needed for proper boot

### Low Priority
7. **Complete 68k Instruction Set**
   - Shift/rotate instructions
   - Multiply/divide
   - Bit manipulation
   - BCD arithmetic

8. **Optimize Performance**
   - Implement decoded instruction cache
   - Add JIT compilation for hot paths
   - Profile and optimize critical loops

## Technical Notes

### ROM Addresses
- **Raw ROM Base**: `0xFFFFFFFF - rom_size + 1` (e.g., `0xFFC8E958` for 3.6MB ROM)
- **ELF Entry**: `base + 0x4100`
- **Decompressed 68k ROM**: Should be at `0xFFC00000` (but not present yet)

### OpenFirmware Calls Observed
- `write` - Text output (working)
- `getprop` - Get device property (working, but some props missing)
- `getproplen` - Get property length (working)
- `finddevice` - Find device node (working for most paths)
- `claim` - Claim memory region (working)

### Missing Components
- `/rtas` device node
- Proper `AAPL,toolbox-parcels` setup
- LZSS decompression trigger
- Exception handlers for null calls

## References

- `docs/M68K_EMULATOR.md` - 68k emulator documentation
- `docs/ROM_DECOMPRESSION.md` - NewWorld ROM decompression notes
- `examples/analyze_raw_rom.rs` - ROM structure analysis tool
- `examples/test_m68k.rs` - 68k emulator test example
- SheepShaver source code - Reference implementation

## Conclusion

We've made excellent progress! The emulator now:
1. ✅ Loads raw NewWorld ROMs correctly
2. ✅ Executes PowerPC boot code from ELF section
3. ✅ Processes OpenFirmware calls
4. ✅ Has a working 68k CPU emulator ready for integration
5. ⚠️ Reaches Mac OS boot sequence (but fails on missing ROM data)

The next critical step is understanding why the ROM decompression isn't happening and fixing the device tree setup to properly advertise the compressed ROM data.
