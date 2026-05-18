# Agent Guidelines for NewtonEmu

## Mac PPC ROM Architecture

### Mac OS ROM Format (NOT ELF!)

**IMPORTANT:** Mac OS ROMs are NOT ELF binaries. Do not assume ELF format or try to parse ELF headers.

**NewWorld ROM Structure:**
1. **CHRP Boot Script** (0x0 - ~0x3800): Text-based OpenFirmware script
2. **Alignment/Padding** (~0x3800 - 0x4000): May contain data or zeros
3. **PowerPC Code** (0x4100+): Native Mac ROM code starts here
   - Entry point detected by finding `mflr r0` (0x7C0802A6) or similar patterns
   - NOT at offset 0x4000 (that's just alignment)

**Key Differences from ELF:**
- No ELF headers (the bytes at 0x4000 that look like ELF are coincidental or embedded data)
- Mac ROM uses its own calling conventions
- Function descriptors are 3-word structures: {code_addr, toc, env}
- r2 register points to globals/TOC structure (not standard ELF RTOC)

**Boot Process:**
1. OpenFirmware loads ROM into memory at 0xFFC00000
2. OF calls ROM entry point (typically around 0xFFC04100)
3. ROM does early initialization
4. ROM may call internal functions or return to OF
5. OF continues boot sequence (loads Mac OS, etc.)

**Memory Layout:**
- ROM: 0xFFC00000 - 0xFFFFFFFF (high memory)
- RAM: 0x00000000 - (configurable, typically 128MB+)
- OpenFirmware stubs: Low memory (0x1000 - 0x5000 range)

## Debugging Guidelines

### Use `dbg!()` not `println!()`
When debugging Rust code, always use `dbg!()` macro instead of `println!()`:

**Good:**
```rust
dbg!(cpu.registers.cr.bits());
dbg!(&some_variable);
```

**Bad:**
```rust
println!("CR: {:?}", cpu.registers.cr.bits());
println!("Value: {}", some_variable);
```

**Why:**
- `dbg!()` prints to stderr, not stdout (won't interfere with test output)
- `dbg!()` includes file location and line number automatically
- `dbg!()` shows the expression being evaluated
- `dbg!()` returns the value, so it can be used inline

## Code Style

- Use Rust 2024 edition
- Follow standard Rust formatting (rustfmt)
- Add comprehensive documentation comments for public APIs
- Include copyright headers on all new files

## Testing

- Write tests for all new functionality
- Use descriptive test names: `test_<feature>_<scenario>`
- Group related tests in the same file
- Add integration tests for cross-module functionality

## Architecture

See `docs/architecture.md` for the overall system design.

## Code Audit Process

When performing a code audit, follow these steps to maintain code quality:

### 1. Find Dead Code

**Check for unused items:**
```bash
cargo build 2>&1 | grep "warning.*never used"
cargo build 2>&1 | grep "warning.*is never read"
cargo build 2>&1 | grep "warning.*never constructed"
```

**Common dead code patterns:**
- Unused functions, structs, constants
- Unused fields in structs
- Unused imports
- Unreachable code paths

**Action:** Remove or document why it's kept (e.g., future use, public API)

### 2. Identify Const Opportunities

**Functions that can be const:**
- Pure functions with no side effects
- Functions that only compute from their parameters
- Getter methods that return references or Copy types
- Constructor functions that build values

**Look for:**
```rust
// Can be const if it only reads fields
fn get_value(&self) -> u32 { self.value }

// Can be const if it's a pure computation
fn add(a: u32, b: u32) -> u32 { a + b }
```

**Make const when possible:**
```rust
const fn get_value(&self) -> u32 { self.value }
const fn add(a: u32, b: u32) -> u32 { a + b }
```

**Note:** Const functions enable compile-time evaluation and const contexts.

### 3. Refactoring Opportunities

**Look for:**
- **Duplicated code:** Extract to helper functions
- **Long functions:** Break into smaller, focused functions (aim for <50 lines)
- **Complex conditionals:** Extract to named boolean functions
- **Magic numbers:** Replace with named constants
- **Overly nested code:** Use early returns or extract functions
- **Mutable state:** Can it be immutable? Use builder patterns?

**Example refactoring:**
```rust
// Before: Magic number
if size > 1024 { ... }

// After: Named constant
const MAX_BUFFER_SIZE: usize = 1024;
if size > MAX_BUFFER_SIZE { ... }
```

### 4. Check for Code Smells

**Common issues:**
- **Unused variables:** Prefix with `_` if intentional: `_unused_var`
- **Large enums/structs:** Consider splitting
- **God objects:** Classes doing too many things
- **Feature envy:** Methods using more of another type than their own
- **Primitive obsession:** Using primitives instead of domain types

### 5. Performance Audit

**Check for:**
- Unnecessary clones (use references instead)
- Inefficient collections usage
- String allocations in hot paths
- Missing `#[inline]` on small hot functions

### 6. Safety Audit

**Review:**
- All `unsafe` blocks (document why they're safe)
- All `unwrap()` calls (replace with proper error handling)
- All `expect()` calls (ensure the message is helpful)
- Integer overflow possibilities

### 7. Documentation Audit

**Ensure:**
- Public APIs have doc comments (`///`)
- Modules have module-level docs (`//!`)
- Complex functions have examples
- Safety requirements are documented
- Error conditions are documented

### Audit Checklist

- [ ] Run `cargo clippy` and address all warnings
- [ ] Run `cargo build` and fix all warnings
- [ ] Check for unused code with `cargo +nightly udeps` (unused dependencies)
- [ ] Look for functions that can be `const`
- [ ] Identify and remove dead code
- [ ] Refactor duplicated code
- [ ] Replace magic numbers with constants
- [ ] Simplify complex functions
- [ ] Remove unnecessary allocations
- [ ] Add missing documentation
- [ ] Review error handling (no unwrap in production paths)
- [ ] Run `cargo test` to ensure nothing breaks
