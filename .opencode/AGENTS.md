# Agent Guidelines for NewtonEmu

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
