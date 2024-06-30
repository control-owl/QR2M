#!/bin/bash

build_linux() {
    echo "Building for Linux..."
    cargo build --release
}

build_windows() {
    echo "Building for Windows..."
    
    rustup target add x86_64-pc-windows-gnu
    
    export PKG_CONFIG_ALLOW_CROSS=1
    export PKG_CONFIG_PATH="/usr/lib/pkgconfig"
    # Point to the GTK4 installation for cross-compilation
    export PKG_CONFIG_SYSROOT_DIR="/opt/gtk/builddir"
    
    # MinGW paths
    export PATH="/usr/x86_64-w64-mingw32/bin:$PATH"
    export LIBRARY_PATH="/usr/x86_64-w64-mingw32/lib"
    export C_INCLUDE_PATH="/usr/x86_64-w64-mingw32/include"
    export CPLUS_INCLUDE_PATH="/usr/x86_64-w64-mingw32/include"
    
    # Set cross-compilation tools
    export CC_x86_64_pc_windows_gnu=x86_64-w64-mingw32-gcc
    export CXX_x86_64_pc_windows_gnu=x86_64-w64-mingw32-g++
    export AR_x86_64_pc_windows_gnu=x86_64-w64-mingw32-ar
    export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc
    
    cargo build --release --target x86_64-pc-windows-gnu
    
    # Optionally, strip the resulting executable
    # strip target/x86_64-pc-windows-gnu/release/qr2m.exe
}

build_macos() {
    echo "Building for macOS..."
    rustup target add x86_64-apple-darwin
    cargo build --release --target x86_64-apple-darwin
}

build_all() {
    build_linux
    build_windows
    build_macos
}

if [ "$1" == "--os" ]; then
    case $2 in
        linux)
            build_linux
            ;;
        win)
            build_windows
            ;;
        mac)
            build_macos
            ;;
        *)
            echo "Invalid OS specified. Use 'linux', 'win', or 'mac'."
            ;;
    esac
else
    build_all
fi
