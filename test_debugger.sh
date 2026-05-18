#!/bin/bash
# Test script for debugger IPC
# Sends JSON commands to the emulator via stdin

set -e

echo "Testing NewtonEmu Debugger IPC"
echo "================================"
echo ""

# Start emulator in background with debugger enabled (headless mode)
# Note: This requires a ROM file. Update the path as needed.
# cargo run --release -- --headless --debugger --rom /path/to/rom.ndrv &
# EMU_PID=$!

# For testing without a ROM, we'll just show what commands would look like:
echo "Example commands to send to emulator stdin:"
echo ""

echo "1. Enable debugger:"
echo '{"command":"enable"}'
echo ""

echo "2. Get CPU state:"
echo '{"command":"get_cpu_state"}'
echo ""

echo "3. Add execution breakpoint at 0x1000:"
echo '{"command":"add_breakpoint","address":4096,"type":"execute"}'
echo ""

echo "4. Get all breakpoints:"
echo '{"command":"get_breakpoints"}'
echo ""

echo "5. Pause execution:"
echo '{"command":"pause"}'
echo ""

echo "6. Step one instruction:"
echo '{"command":"step_into"}'
echo ""

echo "7. Resume execution:"
echo '{"command":"resume"}'
echo ""

echo "8. Remove breakpoint:"
echo '{"command":"remove_breakpoint","address":4096,"type":"execute"}'
echo ""

echo "9. Disable debugger:"
echo '{"command":"disable"}'
echo ""

echo "To test with actual emulator:"
echo "  1. Start emulator: cargo run --release -- --headless --debugger --rom <path>"
echo "  2. Pipe commands: echo '{\"command\":\"get_cpu_state\"}' | cargo run --release -- --headless --debugger --rom <path>"
echo ""

echo "Memory access:"
echo "  - Emulator reports RAM mmap path on startup"
echo "  - Frontend can mmap the file directly for zero-copy access"
echo "  - Example: /tmp/newton_emu_ram_<pid>.bin"
