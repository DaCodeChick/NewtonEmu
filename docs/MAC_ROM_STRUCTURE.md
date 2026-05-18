# Mac OS ROM Structure - NewWorld ROMs

## Discovery: ELF Bootloader in ROM

The NewWorld Mac OS ROM (ROM 2.5.1 from 1999) contains an **actual ELF executable**, which is highly unusual for Mac OS but makes sense in the OpenFirmware/CHRP context.

## ROM File Structure

Total size: 3,612,648 bytes (0x371FE8)

### Section 1: CHRP Boot Script (0x0000 - 0x3FFF)
- Starts with `<CHRP-BOOT>`
- Contains XML-like metadata (COMPATIBLE, DESCRIPTION, ICON)
- Contains Forth boot script (`<BOOT-SCRIPT>`)
- Size: ~16KB

### Section 2: ELF Bootloader (0x4000 - 0x1568F)
- **Offset**: 0x4000 (16,384 bytes)
- **Size**: 0x11690 (71,312 bytes)
- **Format**: Valid ELF executable
- **Architecture**: PowerPC (machine type 20)
- **Type**: EXEC (executable)
- **Endianness**: Big-endian
- **Entry point**: 0x0020B468
- **Program headers**:
  - NOTE segment: vaddr=0x00000000, size=0x2C
  - LOAD segment 1: vaddr=0x00200000, filesz=0xC4A8, memsz=0xC4A8
  - LOAD segment 2: vaddr=0x00100000, filesz=0x5090, memsz=0x15680

### Section 3: LZSS Compressed ROM Image (0x15690 - 0x21DF0F)
- **Offset**: 0x15690 (87,696 bytes)
- **Size**: 0x208880 (2,132,096 bytes)
- **Format**: LZSS compressed data
- **Magic**: `prcl` (0x7072636C)
- Contains the compressed Mac OS ROM Toolbox image

### Section 4: Native PowerPC ROM Code (0x4100+)
- **Entry point**: 0xFFC04100 (when mapped to ROM space)
- **First instruction**: `mfspr r0, lr` (0x7C0802A6)
- This is the traditional Mac ROM initialization code
- NOT ELF format - this is the actual Mac Toolbox

## Why ELF in a Mac ROM?

The NewWorld ROM architecture uses OpenFirmware, which is platform-independent and was designed for multiple operating systems. The boot process is:

1. **OpenFirmware starts** and reads the CHRP boot script
2. **Boot script is Forth code** that:
   - Checks platform compatibility
   - Extracts the ELF bootloader (offset 0x4000)
   - Loads ELF to RAM at load-base
   - Extracts LZSS compressed ROM image (offset 0x15690)
   - Decompresses LZSS data into RAM
   - Creates device tree property `/rom/macos` with `AAPL,toolbox-image,lzss`
   - Calls `init-program` (OF service to set up ELF for execution)
   - Calls `go` to transfer control to the ELF bootloader
3. **ELF bootloader runs** at entry point 0x0020B468
   - This is likely the "Mac OS Nanokernel" or early boot code
   - Sets up the Mac OS environment
   - Initializes the decompressed Toolbox
4. **Mac OS takes over** and continues normal boot

## References

- OpenFirmware IEEE 1275 specification
- CHRP (Common Hardware Reference Platform) boot specification
- PowerPC ELF ABI specification
- Mac OS X / Darwin boot architecture (which evolved from this)

## Implications for Emulation

To properly boot Mac OS, we need to:

1. ✅ Parse CHRP boot script (done)
2. ✅ Implement Forth interpreter (done)
3. ✅ Implement OF client interface (done)
4. ⏳ Execute boot script to extract ELF
5. ⏳ Implement ELF loader
6. ⏳ Implement LZSS decompressor
7. ⏳ Call `init-program` and `go` to start ELF
8. ⏳ Let ELF bootloader initialize Mac OS

This explains why the ROM boot trace only ran ~200 instructions - the code at 0xFFC04100 is not the main entry point! The real boot starts with OpenFirmware executing the Forth boot script.
