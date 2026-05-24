# NewWorld ROM Decompression

## Problem

Mac OS 9 was failing to boot with the error:
```
MacOS: Fatal Error! (0xF3B37FDB)
Official Apple copyright message missing.
```

This error occurred because the emulator was loading NewWorld ROM files without decompressing them.

## NewWorld ROM Format

NewWorld ROMs (used in iMac and later Macs) have a CHRP (Common Hardware Reference Platform) boot script followed by compressed ROM data:

```
+------------------+
| <CHRP-BOOT>      | Boot script (Forth)
| ...              |
| constant values  | lzss-offset, lzss-size, etc.
| boot script      |
+------------------+
| Compressed Data  | LZSS or Parcels format
| ...              |
+------------------+
```

### Boot Script Constants

The boot script defines where the compressed data is located:

```forth
h# 015690 constant lzss-offset
h# 208880 constant lzss-size
```

or for parcels format (Mac OS 9.x):

```forth
h# XXXXXX constant parcels-offset
h# YYYYYY constant parcels-size
```

## Compression Formats

### 1. Plain LZSS Format

Used in earlier NewWorld ROMs. The data at `lzss-offset` is directly LZSS-compressed ROM data.

### 2. Parcels Format (Mac OS 9.x)

Used in Mac OS 9.x ROMs. The data contains multiple "parcels" (packages):

```
Offset 0x00: Magic "prcl" (0x7072636C)
Offset 0x14: First parcel

Each parcel:
  +0x00: next_offset (u32 BE)
  +0x04: parcel_type (u32 BE, FourCC)
  +0x08: lzss_offset (u32 BE, relative to parcel start)
  +0x0C: ... (parcel name, etc.)
```

The ROM data is in the parcel with type `'rom '` (0x726F6D20).

## LZSS Decompression Algorithm

LZSS (Lempel-Ziv-Storer-Szymanski) is a dictionary-based compression:

### Parameters
- Dictionary size: 4096 bytes (0x1000)
- Initial dictionary position: 0xFEE
- Run mask determines operation mode

### Algorithm

1. **Check run mask** (tracks which bits control operations)
   - If < 0x100, read next control byte: `mask = byte | 0xFF00`
   
2. **Process based on lowest bit**:
   - **Bit = 1**: Verbatim copy
     - Read 1 byte directly
     - Add to dictionary and output
   
   - **Bit = 0**: Dictionary copy
     - Read 2 bytes: `[offset_low, offset_high_and_length]`
     - `dict_offset = offset_low | ((offset_high_and_length & 0xF0) << 4)`
     - `run_length = (offset_high_and_length & 0x0F) + 3`
     - Copy `run_length` bytes from dictionary at `dict_offset`

3. **Update dictionary**:
   - Add each output byte to dictionary
   - Dictionary position wraps at 0x1000

### Example

```
Input: [0xFF, 0x41, 0x42, 0x00, 0xF0, 0x02, ...]
       ^^^^  ^^^^  ^^^^  ^^^^  ^^^^  ^^^^
       mask  'A'   'B'   dict  dict  ...
                         ref   ref
```

- `0xFF` → run_mask = 0xFFFF (all verbatim)
- `0x41` → output 'A', add to dict
- `0x42` → output 'B', add to dict
- `0x00 0xF2` → copy from dict[0x0F0], length 5

## Implementation

### Finding Constants

```rust
fn find_boot_constant(script: &str, name: &str) -> Option<usize> {
    // Look for "h# XXXXXX constant name"
    let pattern = format!("constant {}", name);
    if let Some(pos) = script.find(&pattern) {
        // Extract hex value before "constant"
        // ...
    }
}
```

### Decoding Parcels

```rust
fn decode_parcels(data: &[u8]) -> Result<Vec<u8>> {
    let mut parcel_offset = 0x14; // First parcel
    
    while parcel_offset != 0 {
        let next_offset = read_u32_be(&data[parcel_offset..]);
        let parcel_type = read_u32_be(&data[parcel_offset + 4..]);
        
        if parcel_type == 0x726F6D20 { // 'rom '
            let lzss_offset = read_u32_be(&data[parcel_offset + 8..]);
            // Decompress LZSS data...
            return Ok(decoded);
        }
        
        parcel_offset = next_offset;
    }
}
```

### LZSS Decompression

```rust
fn decode_lzss(src: &[u8]) -> Vec<u8> {
    let mut dest = Vec::new();
    let mut dict = [0u8; 0x1000];
    let mut run_mask = 0u16;
    let mut dict_idx = 0xfee;
    let mut src_idx = 0;
    
    loop {
        if run_mask < 0x100 {
            if src_idx >= src.len() { break; }
            run_mask = (src[src_idx] as u16) | 0xff00;
            src_idx += 1;
        }
        
        let bit = (run_mask & 1) != 0;
        run_mask >>= 1;
        
        if bit {
            // Verbatim copy
            let c = src[src_idx];
            src_idx += 1;
            dict[dict_idx] = c;
            dict_idx = (dict_idx + 1) & 0xfff;
            dest.push(c);
        } else {
            // Dictionary copy
            let b0 = src[src_idx] as usize;
            let b1 = src[src_idx + 1] as usize;
            src_idx += 2;
            
            let dict_offset = b0 | ((b1 & 0xf0) << 4);
            let run_length = (b1 & 0x0f) + 3;
            
            for _ in 0..run_length {
                let c = dict[dict_offset];
                dict[dict_idx] = c;
                dict_idx = (dict_idx + 1) & 0xfff;
                dest.push(c);
            }
        }
    }
    
    dest
}
```

## Results

### Before Decompression
- Raw ROM file size: ~2-3 MB
- Mac OS error: "Official Apple copyright message missing"
- Boot failed immediately after ROM initialization

### After Decompression
- Decoded ROM size: 4,194,315 bytes (4 MB)
- Copyright error: ✅ GONE
- Mac OS boot progresses to system initialization
- Now needs additional device/system emulation

## Test Results

```
$ cargo run
[INFO] NewWorld ROM: lzss-offset=0x15690, lzss-size=0x208880
[INFO] ROM uses parcels format - decoding parcels
[INFO] Found 'rom ' parcel: LZSS at 0xD044, size 0x1CFDD8
[INFO] Loaded ROM: 4194315 bytes (NewWorld) from roms/1999-09-17 - Mac OS ROM 2.5.1.rom
[INFO] ✓ Set AAPL,toolbox-parcels property: rom_base=0xFFC00000, size=4194315
```

No more copyright error! 🎉

## References

- SheepShaver source: `src/rom_patches.cpp`
  - `decode_lzss()` function
  - `decode_parcels()` function
- LZSS algorithm: Lempel-Ziv-Storer-Szymanski compression
- NewWorld ROM format: Apple CHRP boot specification

## Files Modified

- `crates/newton-core/src/rom.rs`:
  - Added `decode_lzss()`
  - Added `decode_parcels()`
  - Added `decode_newworld_rom()`
  - Added `find_boot_constant()`
  - Updated `load_from_file()` to decompress NewWorld ROMs
