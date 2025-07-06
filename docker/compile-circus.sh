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

mkdir -p "$CIRCUS"
mkdir -p "$LOG_DIR"

cd /home/QR2M/compile-circus

echo "Set PKG_CONFIG_PATH"
export PKG_CONFIG_PATH="$STATIC_DIR/lib:$STATIC_DIR/lib/pkgconfig:/usr/lib/pkgconfig:/usr/share/pkgconfig:/usr/lib/x86_64-linux-musl/pkgconfig:/usr/local/lib/pkgconfig"
echo "PKG_CONFIG_PATH=$PKG_CONFIG_PATH"


echo "Set environment variables for build"
export PKG_CONFIG_ALLOW_CROSS=1
export CFLAGS="-I$STATIC_DIR/include -static"
export LDFLAGS="-L$STATIC_DIR/lib -static"
export OPENSSL_DIR=/usr
export OPENSSL_LIB_DIR=/usr/lib
export OPENSSL_INCLUDE_DIR=/usr/include
export OPENSSL_STATIC=1
export RUSTFLAGS="-C target-feature=+crt-static -C link-arg=-L/usr/lib -C link-arg=-L$STATIC_DIR/lib/pkgconfig -C link-arg=-L$STATIC_DIR/lib -C link-arg=-lssl -C link-arg=-lcrypto -C link-arg=-static"
#export RUSTFLAGS="-C target-feature=+crt-static -C link-arg=-L$STATIC_DIR/lib -C link-arg=-static"


echo "Checking pkg-config for dependencies"
for pkg in gtk4 libadwaita-1; do
  echo "Checking $pkg..."
  pkg-config --modversion "$pkg" 2>&1 | tee -a "$LOG_DIR/pkg_config_check.log" || { echo "Error: $pkg not found"; exit 1; }
done

echo "Listing .pc files in $STATIC_DIR/lib/pkgconfig:"
ls -l "$STATIC_DIR/lib/pkgconfig" | tee -a "$LOG_DIR/pkg_config_list.log"


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

echo "Listing build directory:"
ls -l "$BUILD_PATH/release"



# PAUSE
#cargo test --release --locked - --no-fail-fast --target "$TARGET" --features "$FEATURES" 2>&1 | tee "$LOG_DIR/qr2m_test.log"
#STATUS=$?
#if [ "$STATUS" -ne 0 ]; then
#  cat $LOG_DIR/qr2m_test.log
#  echo "CARGO TEST QR2M FAIL"
#  exit 1
#fi
#
#echo "Listing build directory:"
#ls -l "$BUILD_PATH/release"
#
#echo "Checking binary:"
#export BIN="$BUILD_PATH/release/$APP_NAME"
#[ -f "$BIN" ] || { echo "Error: Binary not found at $BIN"; exit 1; }
#file "$BIN" 2>&1 | tee "$LOG_DIR/qr2m_file_check.log"
#ldd "$BIN" 2>&1 | tee "$LOG_DIR/qr2m_ldd_check.log"
#chmod +x "$BIN"
#
#echo "Creating output binary"
#if [ "$OUTPUT" = "true" ]; then
#  echo "Copying files to $OUTPUT_DIR..."
#  mkdir -p "$OUTPUT_DIR"
#  cp "$BIN" "$OUTPUT_DIR" || { echo "Error: Failed to copy $BIN to $OUTPUT_DIR"; exit 1; }
#  #chown 1001:1001 "$OUTPUT_DIR/$APP_NAME" || { echo "Error: Failed to change ownership of $OUTPUT_DIR/$APP_NAME"; exit 1; }
#
#  echo "Listing output directory:"
#  ls -l "$OUTPUT_DIR"
#fi