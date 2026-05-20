#!/bin/bash
# NewtonEmu Frontend Build Script

set -e

echo "===================================================="
echo "NewtonEmu Frontend Build Script"
echo "===================================================="
echo ""

# Check for Qt 6
if ! command -v qmake6 &> /dev/null && ! command -v qmake &> /dev/null; then
    echo "Error: Qt 6 not found!"
    echo ""
    echo "Please install Qt 6:"
    echo "  Ubuntu/Debian: sudo apt install qt6-base-dev qt6-tools-dev"
    echo "  Fedora: sudo dnf install qt6-qtbase-devel qt6-qttools-devel"
    echo "  macOS: brew install qt@6"
    echo ""
    exit 1
fi

# Check for CMake
if ! command -v cmake &> /dev/null; then
    echo "Error: CMake not found!"
    echo ""
    echo "Please install CMake:"
    echo "  Ubuntu/Debian: sudo apt install cmake"
    echo "  Fedora: sudo dnf install cmake"
    echo "  macOS: brew install cmake"
    echo ""
    exit 1
fi

# Create build directory
mkdir -p build
cd build

# Configure
echo "Configuring..."
cmake .. "$@"

# Build
echo ""
echo "Building..."
make -j$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)

echo ""
echo "===================================================="
echo "Build complete!"
echo "===================================================="
echo ""
echo "Binary: $(pwd)/bin/NewtonEmu"
echo ""
echo "To run:"
echo "  cd build"
echo "  ./bin/NewtonEmu"
echo ""
