#!/bin/bash

build_linux() {
    echo "Building for Linux..."
    cargo build --release
}

build_windows() {
    echo "Building for Windows..."
    
    rustup target add x86_64-pc-windows-gnu
    
    export PATH="/usr/x86_64-w64-mingw32/bin:$PATH"
    export PKG_CONFIG_PATH="/opt/gtk4-win64/lib/pkgconfig"  # Adjust this path to where GTK4 .pc files are located
    export PKG_CONFIG_SYSROOT_DIR="/usr/x86_64-w64-mingw32/sys-root/mingw"
    export C_INCLUDE_PATH="/usr/x86_64-w64-mingw32/include"
    export CPLUS_INCLUDE_PATH="/usr/x86_64-w64-mingw32/include"
    export LIBRARY_PATH="/opt/gtk4-win64/lib"  # Adjust this path to where GTK4 libraries (.a files) are located
    export LD_LIBRARY_PATH="/opt/gtk4-win64/lib"  # Adjust this path to where GTK4 libraries (.so files) are located
    
    # export RUSTFLAGS="-L /opt/gtk4-win64/lib"
    
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
