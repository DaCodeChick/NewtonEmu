# MMU Integration TODO

## Current Status

### ✅ Completed
- PowerPC MMU core implementation (`crates/newton-cpu/src/mmu.rs`)
  - BAT (Block Address Translation) registers and logic
  - Segment register translation
  - Page table structures (HTAB, PTE, PTEG)
  - TLB cache with invalidation (`tlbie`, `tlbia`, `tlbsync`)
  - Primary and secondary hash chains
  - PTE matching (VSID, API, hash function indicator)
  - Page protection checks
- SPR (Special Purpose Register) integration
  - `mtspr`/`mfspr` for BAT registers (IBAT0-3, DBAT0-3)
  - `mtspr`/`mfspr` for SDR1
- Memory interface split
  - `MemoryInterface`: virtual address operations for CPU
  - `PhysicalMemory`: physical address operations for MMU
  - `Memory` struct implements both traits
- MMU state integrated into `Registers`
- MMU can read page tables from physical memory via `lookup_pte()`

### ⏳ TODO

1. **CPU execution integration**
   - Design: How should load/store instructions invoke MMU translation?
   - Options:
     a. Inline translation in each load/store function
     b. Wrapper at `Cpu::step()` level
     c. Separate "translating memory" adapter passed to interpreter
   - Need to handle both `MemoryInterface` and `PhysicalMemory` requirements
   
2. **MMU exception handling**
   - Add DSI (Data Storage Interrupt) for data access violations
   - Add ISI (Instruction Storage Interrupt) for instruction fetch violations
   - Add DAR (Data Address Register) and DSISR (DSI Status Register)
   - Integrate with existing exception system in `crates/newton-cpu/src/exceptions.rs`

3. **Testing**
   - Unit tests for MMU translation with mock memory
   - Integration test with Mac OS 9.2.2 boot
   - Verify BAT setup during OpenFirmware
   - Verify page table usage by Mac OS kernel

## Architecture Notes

### Memory Access Flow
```
CPU Instruction
  └─> Calculate Effective Address (EA)
       └─> MMU Translation (if MSR[DR/IR] = 1)
            ├─> Check BAT registers
            ├─> Check TLB cache
            └─> Walk page tables (primary/secondary hash)
                 └─> Physical Memory Access
```

### Current Design Challenge

The CPU's `step()` method takes `memory: &dyn MemoryInterface`, but MMU translation requires:
1. Access to MMU state (in `Registers`)
2. Ability to read page tables via `PhysicalMemory`
3. Translate virtual → physical before calling memory operations

**Possible Solutions:**

**Option A: Inline Translation in Load/Store**
```rust
// In loadstore.rs
pub fn lwz(regs: &mut Registers, memory: &dyn PhysicalMemory, rt: u8, ra: u8, d: i16) -> Result<()> {
    let vaddr = effective_address(regs, ra, d as i32);
    let paddr = regs.mmu.translate_data(vaddr, &regs.sr, regs.msr, false, memory)?;
    let value = memory.read_u32(paddr)?;  // Physical read
    regs.gpr[rt as usize] = value;
    Ok(())
}
```
- ✅ Simple, direct
- ❌ Requires changing all load/store functions
- ❌ Requires `memory` to be `PhysicalMemory` not `MemoryInterface`

**Option B: Translating Memory Adapter**
```rust
struct TranslatingMemory<'a, M: PhysicalMemory> {
    physical: &'a M,
    mmu: &'a mut Mmu,
    sr: &'a [u32; 16],
    msr: u32,
}

impl<'a, M: PhysicalMemory> MemoryInterface for TranslatingMemory<'a, M> {
    fn read_u32(&self, vaddr: u32) -> Result<u32> {
        let paddr = self.mmu.translate_data(vaddr, self.sr, self.msr, false, self.physical)?;
        self.physical.read_u32(paddr)
    }
    // ...
}
```
- ✅ No changes to load/store functions
- ✅ Clean separation of concerns
- ❌ Borrow checker complexity (mutable MMU, immutable SR/MSR)
- ❌ Lifetime management

**Option C: Two-Phase Memory Interface**
```rust
// Cpu::step() creates adapter
let phys_mem: &Memory = ...;
let translator = MemoryTranslator {
    cpu: &mut self,
    memory: phys_mem,
};
self.step_interpreter(&translator)?;
```
- ✅ Encapsulates translation logic
- ✅ Interpreter unchanged
- ❌ Still has borrow checker issues

**Recommended: Option A with PhysicalMemory**

After analysis, Option A is cleanest:
1. Change all load/store to take `&dyn PhysicalMemory` instead of `&dyn MemoryInterface`
2. Have them call `translate_data()` explicitly before physical access
3. Change `Memory::read/write` to always use physical addresses
4. Add helper methods on `Cpu` or `Registers` for easy translation access

This makes the distinction between virtual and physical addresses explicit throughout the code.

## Implementation Steps

1. ✅ Implement MMU core with all translation logic
2. ✅ Add PhysicalMemory trait for page table reads
3. ⏳ Refactor load/store instructions to use inline MMU translation
4. ⏳ Add MMU exception types (DSI, ISI) and handler integration
5. ⏳ Add DAR and DSISR registers
6. ⏳ Test with Mac OS 9.2.2 boot sequence
7. ⏳ Optimize: measure TLB hit rate, tune TLB size

## Testing Strategy

### Unit Tests
- BAT translation: various address ranges, protection bits
- Segment register translation: all 16 segments
- Page table lookup: primary hash, secondary hash, no match
- TLB: hit, miss, invalidation
- Protection violations: read-only pages, no-access pages

### Integration Tests  
- Boot Mac OS 9.2.2 and verify MMU setup
- Trace OpenFirmware BAT configuration
- Verify kernel sets up page tables correctly
- Test context switch (segment register changes)
- Test memory-mapped I/O with BAT vs. page tables

## References
- PowerPC 32-bit Architecture Book III (Operating Environment)
- IBM PowerPC 750 (G3) User Manual - Chapter 7
- Motorola MPC7450 (G4) User Manual - Chapter 5
- Mac OS 9.2.2 kernel MMU initialization code (via ROM analysis)
