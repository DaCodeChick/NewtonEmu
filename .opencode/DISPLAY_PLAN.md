# Display Implementation Plan

## Goal
Add visual output and optional debugging UI to NewtonEmu

## Architecture (Already Designed)

### Window 1: Emulation Display (Pure wgpu)
- Shows Mac framebuffer only
- No UI overlay
- Full-screen capable
- Always visible when emulator runs

### Window 2: Debug UI (egui + wgpu)
- **Completely separate window**
- **Optional and togglable**
- Spawned via F12 or command-line flag
- Can be closed without affecting emulation

## Implementation Steps

### Phase 1: Basic Display Window (~1-2 hours)

**Create `crates/newton-ui/` crate:**
- Display window using winit + wgpu
- Render framebuffer texture to screen
- Handle window events (close, resize)
- Basic keyboard input

**Files to create:**
- `crates/newton-ui/Cargo.toml`
- `crates/newton-ui/src/lib.rs`
- `crates/newton-ui/src/display.rs`

**Create main binary:**
- `src/main.rs` - CLI entry point
- Parse arguments (ROM path, debug flag, etc.)
- Create emulator + display
- Run event loop

### Phase 2: Debug Window (~2-3 hours)

**Add egui debug window:**
- `crates/newton-ui/src/debug.rs`
- Separate window from display
- Toggle with F12 key
- Command-line flag: `--debug` or `-d`

**Debug panels:**
1. CPU Registers
   - PC, LR, CTR, MSR, CR
   - GPRs (r0-r31)
   - Formatting: hex, decimal

2. Memory Viewer
   - Hex dump view
   - Address input
   - Follow PC option
   - ASCII sidebar

3. Execution Stats
   - Instructions/second
   - Total executed
   - Branch/load/store counts

4. Step Controls
   - Pause/Resume
   - Single step
   - Step over
   - Reset

### Phase 3: Integration (~30 mins)

**Update emulator:**
- Add `is_paused()` state
- Add `single_step()` method
- Add `get_stats()` for debug display

**Event loop:**
```rust
loop {
    // Handle window events
    event_loop.poll();
    
    // Update emulator (if not paused)
    if !emulator.is_paused() {
        emulator.step()?;
    }
    
    // Render display window
    display.render(emulator.framebuffer());
    
    // Render debug window (if open)
    if let Some(debug) = &mut debug_window {
        debug.render(&emulator);
    }
    
    // Handle hotkeys
    if pressed_f12 {
        debug_window = debug_window.toggle();
    }
}
```

## Dependencies to Add

```toml
# newton-ui/Cargo.toml
[dependencies]
winit = "0.30"           # Window management
wgpu = "23.0"            # Graphics
egui = "0.29"            # Debug UI
egui-wgpu = "0.29"       # egui + wgpu integration
pollster = "0.4"         # Async runtime for wgpu
```

## Configuration

```rust
// CLI args (clap)
struct Args {
    /// ROM file path
    rom: PathBuf,
    
    /// Show debug window on startup
    #[arg(short, long)]
    debug: bool,
    
    /// Start paused
    #[arg(short, long)]
    paused: bool,
    
    /// RAM size in MB
    #[arg(long, default_value = "256")]
    ram: usize,
}
```

## Success Criteria

### Display Window:
- ✅ Shows framebuffer content
- ✅ Window can be resized
- ✅ Handles close events
- ✅ Renders at 60 FPS

### Debug Window:
- ✅ Can be toggled with F12
- ✅ Shows CPU registers updating in real-time
- ✅ Memory viewer works
- ✅ Can pause/resume execution
- ✅ Can single-step through code
- ✅ Execution stats update live

### Integration:
- ✅ Emulator runs in display mode
- ✅ Debug window is optional (--debug flag)
- ✅ Both windows work independently
- ✅ Closing debug window doesn't stop emulation
- ✅ Closing display window stops emulation

## Testing

**Test 1: Headless mode (existing)**
```bash
cargo run --example boot_rom
cargo run --example test_rom_exec 0xFFC06000 1000
```

**Test 2: Display only**
```bash
cargo run -- roms/Mac\ OS\ ROM\ 3.0.rom
# Should show blank display (framebuffer not written yet)
# Should execute ROM code
# Should respond to close button
```

**Test 3: Display + Debug**
```bash
cargo run -- --debug roms/Mac\ OS\ ROM\ 3.0.rom
# Should show both windows
# Debug should show registers updating
# F12 should close debug window
# F12 again should reopen it
```

**Test 4: Pause/Step**
```bash
cargo run -- --debug --paused roms/Mac\ OS\ ROM\ 3.0.rom
# Should start paused
# Step button should advance one instruction
# Resume should run continuously
```

## Future Enhancements (Later)

- Breakpoints
- Watchpoints
- Disassembly view
- Save states
- Performance profiler
- Network stats
- Device status

## Current Session Achievement

✅ 50,000 ROM instructions executing  
✅ 100% valid PowerPC code  
✅ Ready for visualization  

Next session: Build the display!
