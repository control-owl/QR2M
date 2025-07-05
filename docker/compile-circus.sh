#!/bin/bash
set -e

APP_NAME="QR2M"
TARGET="x86_64-unknown-linux-musl"
FEATURES="dev"
BUILD_PATH="target/$TARGET"
OUTPUT_DIR="$BUILD_PATH/release"
OUTPUT="false"

CIRCUS="/home/QR2M/compile-circus"
LOG_DIR="$CIRCUS/LOG"
STATIC_DIR="$CIRCUS/STATIC"

mkdir -p $CIRCUS
mkdir -p $LOG_DIR

cd /home/QR2M/compile-circus

echo "Set PKG_CONFIG_PATH"
export PKG_CONFIG_PATH="$STATIC_DIR/lib/pkgconfig:/usr/lib/pkgconfig:/usr/share/pkgconfig:/usr/lib/x86_64-linux-musl/pkgconfig:/usr/local/lib/pkgconfig"
echo "PKG_CONFIG_PATH=$PKG_CONFIG_PATH"


echo "Set environment variables for build"
export PKG_CONFIG_ALLOW_CROSS=1
export CFLAGS="-static"
export LDFLAGS="-static"
export OPENSSL_DIR=/usr
export OPENSSL_LIB_DIR=/usr/lib
export OPENSSL_INCLUDE_DIR=/usr/include
export OPENSSL_STATIC=1
export RUSTFLAGS="-C target-feature=+crt-static -C link-arg=-L/usr/lib -C link-arg=-L$STATIC_DIR -C link-arg=-lssl -C link-arg=-lcrypto -C link-arg=-static"


echo "Capturing .pc file paths..."
GTK4=$(apk info -L gtk4.0-dev | grep -E '/gtk4\.pc$' | sed "s|^|/|")
#ADWAITA=$(apk info -L libadwaita-dev | grep -E '/libadwaita-1\.pc$' | sed "s|^|/|")
echo "GTK4=$GTK4"
#echo "ADWAITA=$ADWAITA"
[ -n "$GTK4" ] || { echo "Error: gtk4.pc not found in gtk4.0-dev"; exit 1; }
#[ -n "$ADWAITA" ] || { echo "Error: libadwaita-1.pc not found in libadwaita-dev"; exit 1; }
#
#
echo "Renaming .pc files..."
cp "$GTK4" "$(dirname "$GTK4")/gtk-4.pc" || { echo "Error: Failed to rename gtk4.pc"; exit 1; }
#cp "$ADWAITA" "$(dirname "$ADWAITA")/libadwaita-1.0.pc" || { echo "Error: Failed to rename libadwaita-1.pc"; exit 1; }


echo "Cloning project..."
git clone https://github.com/control-owl/QR2M QR2M
cd QR2M

cargo build --release --target "$TARGET" --features "$FEATURES" --locked -vv 2>&1 | tee "$LOG_DIR/qr2m_build.log"
STATUS=$?
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/qr2m_build.log
  echo "CARGO BUILD QR2M FAIL"
  exit 1
fi

cargo test --release --locked - --no-fail-fast --target "$TARGET" --features "$FEATURES" 2>&1 | tee "$LOG_DIR/qr2m_test.log"
STATUS=$?
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/qr2m_test.log
  echo "CARGO TEST QR2M FAIL"
  exit 1
fi

echo "Listing build directory:"
ls -l "$BUILD_PATH"

echo "Checking binary:"
export BIN="$BUILD_PATH/$APP_NAME"
[ -f "$BIN" ] || { echo "Error: Binary not found at $BIN"; exit 1; }
file "$BIN" 2>&1 | tee "$LOG_DIR/qr2m_file_check.log"
ldd "$BIN" 2>&1 | tee "$LOG_DIR/qr2m_ldd_check.log"
chmod +x "$BIN"

echo "Creating output binary"
if [ "$OUTPUT" = "true" ]; then
  echo "Copying files to $OUTPUT_DIR..."
  mkdir -p "$OUTPUT_DIR"
  cp "$BIN" "$OUTPUT_DIR" || { echo "Error: Failed to copy $BIN to $OUTPUT_DIR"; exit 1; }
#  chown 1001:1001 "$OUTPUT_DIR/$APP_NAME" || { echo "Error: Failed to change ownership of $OUTPUT_DIR/$APP_NAME"; exit 1; }

  echo "Listing output directory:"
  ls -l "$OUTPUT_DIR"
fi



#echo "Set PKG_CONFIG_PATH"
#export PKG_CONFIG_PATH="/usr/lib/pkgconfig:/usr/share/pkgconfig:/usr/lib/x86_64-linux-musl/pkgconfig:/usr/local/lib/pkgconfig"
#GTK_PC_PATH=$(dirname "$GTK4")
#LIBADWAITA_PC_PATH=$(dirname "$ADWAITA")
#export PKG_CONFIG_PATH="$GTK_PC_PATH:$LIBADWAITA_PC_PATH:$PKG_CONFIG_PATH"
#echo "PKG_CONFIG_PATH=$PKG_CONFIG_PATH"
#
#
#echo "Checking pkg-config versions..."
#pkg-config --modversion gtk-4.0 || { echo "Error: gtk-4.0 not found"; exit 1; }
#pkg-config --modversion libadwaita-1.0 || { echo "Error: libadwaita-1.0 not found"; exit 1; }
#pkg-config --libs --cflags openssl || { echo "Error: OpenSSL pkg-config not found"; exit 1; }
#
#
#echo "Install Rust MUSL target"
#rustup target add x86_64-unknown-linux-musl
#
#
#echo "Set environment variables for build"
#export PKG_CONFIG_ALLOW_CROSS=1
##export CFLAGS="-I/usr/include"
##export LDFLAGS="-L/usr/lib -L/usr/lib/x86_64-linux-musl"
## export CFLAGS="-static -O2 -fPIC"
#export CFLAGS="-static"
#export LDFLAGS="-static"
#export OPENSSL_DIR=/usr
#export OPENSSL_LIB_DIR=/usr/lib
#export OPENSSL_INCLUDE_DIR=/usr/include
#export OPENSSL_STATIC=1
#export RUSTFLAGS="-C target-feature=+crt-static -C link-arg=-L/usr/lib -C link-arg=-lssl -C link-arg=-lcrypto -C link-arg=-static"
## export RUSTFLAGS="-C target-feature=+crt-static -C linker=musl-gcc"
#
#
#echo "Building project..."
#cd /home/QR2M/compile-circus
#git clone https://github.com/control-owl/QR2M
#cd QR2M
#cargo build --release --target "$TARGET" --features "$FEATURES" --locked -vv && echo "Cargo build done" || exit 1
## cargo test --release --locked - --no-fail-fast --target "$TARGET" --features "$FEATURES" && echo "Cargo test done"
#
#
#echo "Listing build directory:"
#ls -l "$BUILD_PATH"
#
#
#echo "Checking binary:"
#export BIN="$BUILD_PATH/$APP_NAME"
#[ -f "$BIN" ] || { echo "Error: Binary not found at $BIN"; exit 1; }
#file "$BIN"
#ldd "$BIN"
#chmod +x "$BIN"
#
#
#if [ "$OUTPUT" = "true" ]; then
#  echo "Copying files to $OUTPUT_DIR..."
#  mkdir -p "$OUTPUT_DIR"
#  cp "$BIN" "$OUTPUT_DIR" || { echo "Error: Failed to copy $BIN to $OUTPUT_DIR"; exit 1; }
##  chown 1001:1001 "$OUTPUT_DIR/$APP_NAME" || { echo "Error: Failed to change ownership of $OUTPUT_DIR/$APP_NAME"; exit 1; }
#
#  echo "Listing output directory:"
#  ls -l "$OUTPUT_DIR"
#fi
#
#
## ALL GTK4 deps in Arch Linux
## Dependencies (61)
## adwaita-fonts
## adwaita-icon-theme
## at-spi2-core
## bash
## cairo
## dconf
## desktop-file-utils
## fontconfig
## fribidi
## gcc-libs
## gdk-pixbuf2
## glib2
## glibc
## graphene
## gst-plugins-bad-libs
## gst-plugins-base-libs
## gstreamer
## gtk-update-icon-cache
## harfbuzz
## iso-codes
## libcloudproviders
## libcolord
## libcups
## libegl (libglvnd)
## libepoxy
## libgl (libglvnd)
## libjpeg-turbo
## libpng
## librsvg
## libtiff
## libx11
## libxcursor
## libxdamage
## libxext
## libxfixes
## libxi
## libxinerama
## libxkbcommon
## libxrandr
## libxrender
## pango
## shared-mime-info
## tinysparql
## vulkan-icd-loader
## wayland
## evince (optional) - Default print preview command
## cantarell-fonts (make)
## docbook-xsl (make)
## gi-docgen (make)
## git (make)
## glib2-devel (make)
## gobject-introspection (make)
## hicolor-icon-theme (make)
## libsysprof-capture (make)
## meson (make)
## python-docutils (make)
## python-gobject (make)
## sassc (make)
## shaderc (make)
## vulkan-headers (make)
## wayland-protocols (make)