# MMU Integration TODO

## Current Status
✅ MMU core implementation complete in `crates/newton-cpu/src/mmu.rs`
✅ BAT registers (IBAT0-3, DBAT0-3) accessible via mtspr/mfspr
✅ TLB management (tlbie, tlbia, tlbsync) implemented
✅ Page table structures (PTE, HTAB) defined
✅ Translation logic (BAT → TLB → Page Tables) implemented

## Remaining Work

### 1. Integrate MMU with Memory Interface
**Files to modify:**
- `crates/newton-core/src/memory.rs`
- `crates/newton-cpu/src/mmu.rs`

**Changes needed:**
1. Split Memory interface into:
   - `PhysicalMemory` - raw physical address access (for MMU page table reads)
   - `VirtualMemory` - wraps PhysicalMemory + MMU for translated access

2. Update `Mmu::lookup_pte()` to accept a `PhysicalMemory` interface:
   ```rust
   fn lookup_pte(&self, memory: &dyn PhysicalMemory, ...) -> Option<PageTableEntry> {
       // Read PTE from physical memory at pteg_addr
       let pte_word0 = memory.read_u32_phys(pteg_addr)?;
       let pte_word1 = memory.read_u32_phys(pteg_addr + 4)?;
       // ...
   }
   ```

3. Create `VirtualMemory` wrapper:
   ```rust
   pub struct VirtualMemory {
       physical: Arc<Memory>,
       mmu: Arc<RwLock<Mmu>>,
       cpu_regs: Arc<RwLock<Registers>>, // For SR and MSR
   }
   
   impl MemoryInterface for VirtualMemory {
       fn read_u32(&self, vaddr: u32) -> Result<u32> {
           let regs = self.cpu_regs.read();
           let mut mmu = self.mmu.write();
           let paddr = mmu.translate_data(vaddr, &regs.sr, regs.msr.bits(), false)?;
           self.physical.read_u32_phys(paddr)
       }
       // ... similar for other operations
   }
   ```

### 2. Update Emulator to use MMU
**File:** `crates/newton-core/src/emulator.rs`

**Changes:**
1. Pass `VirtualMemory` to CPU instead of raw `Memory`
2. Ensure CPU register state is shared with VirtualMemory
3. Handle MMU exceptions (page faults, protection violations)

### 3. Handle MMU Exceptions
**Files to modify:**
- `crates/newton-cpu/src/exceptions.rs`
- `crates/newton-cpu/src/mmu.rs`

**New exception types:**
- `DataStorageInterrupt` (DSI) - data access violation
- `InstructionStorageInterrupt` (ISI) - instruction fetch violation
- Set DAR (Data Address Register) with faulting address
- Set DSISR (Data Storage Interrupt Status Register) with error info

### 4. Memory Protection
**File:** `crates/newton-cpu/src/mmu.rs`

**Add protection checks:**
```rust
fn check_protection(&self, pp: u8, msr_pr: bool, is_write: bool) -> Result<()> {
    // PP bits:
    // 00 - no access
    // 01 - read-only
    // 10 - read/write
    // 11 - read-only (both user and supervisor)
    
    match pp {
        0b00 => bail!("Page protection: no access"),
        0b01 | 0b11 if is_write => bail!("Page protection: read-only"),
        _ => Ok(())
    }
}
```

### 5. Test with Mac OS 9.2.2
**Test procedure:**
1. Enable MMU translation (currently disabled)
2. Boot Mac OS 9.2.2 CD
3. Watch for BAT register setup in early boot
4. Verify page table initialization (SDR1 write)
5. Check for page faults and ensure they're handled
6. Verify memory protection violations are caught

**Expected behavior:**
- ROM should be mapped via IBAT/DBAT (no translation)
- RAM should use page tables after OS initialization
- Mac OS should get past the copyright fatal error

### 6. Performance Optimization
Once working, optimize:
- TLB hit rate (currently using HashMap, could use direct-mapped array)
- BAT check ordering (put most-used BATs first)
- Cache MSR[IR/DR] checks
- Pre-translate hot addresses

## Architecture Notes

### Translation Flow
```
Virtual Address (32-bit)
    |
    v
MSR[IR/DR] enabled? --NO--> Physical Address
    |
    YES
    v
Try IBAT/DBAT (4 entries each)
    |
    MISS
    v
Check TLB cache (HashMap)
    |
    MISS
    v
Segment Register (SR[top 4 bits])
    |
    v
Primary Hash (VSID ^ page_index)
    |
    v
Read PTEG from physical memory (via SDR1)
    |
    MISS
    v
Secondary Hash (!primary_hash)
    |
    v
Read PTEG from physical memory
    |
    FOUND
    v
Physical Address + cache in TLB
```

### Register Usage
- **MSR[IR]** (bit 5): Instruction address translation enable
- **MSR[DR]** (bit 4): Data address translation enable  
- **MSR[PR]** (bit 14): Problem state (user mode)
- **SR0-SR15**: Segment registers (256MB each)
- **SDR1**: Page table base and size
- **IBAT0-3**: Instruction BATs (large block mapping)
- **DBAT0-3**: Data BATs (large block mapping)
- **DAR**: Data Address Register (faulting address)
- **DSISR**: DSI status (fault type)

### Memory Layout (Mac OS 9)
Typical Mac OS 9 memory map with MMU:
- `0x00000000-0x0FFFFFFF`: Application RAM (256MB, via page tables)
- `0xF0000000-0xF7FFFFFF`: I/O space (via DBAT)
- `0xFFC00000-0xFFFFFFFF`: ROM (via IBAT/DBAT, 4MB)

ROM is usually mapped with:
- IBAT0U = 0xFFC0003F (BEPI=0xFFC00000, BL=0x3=512KB, Vs=1, Vp=1)
- IBAT0L = 0xFFC00032 (BRPN=0xFFC00000, WIMG=0x3, PP=0x2=read-only)
