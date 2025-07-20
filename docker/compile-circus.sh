#!/bin/bash
# authors = ["Control Owl <qr2m[at]r-o0-t[dot]wtf>"]
# license = "CC-BY-NC-ND-4.0  [2023-2025]  Control Owl"
#
# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

set -e
set -x
set -o pipefail

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
mkdir -p "$STATIC_DIR"

cd /home/QR2M/compile-circus

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.
echo "PKG_CONFIG_PATH=$PKG_CONFIG_PATH"

echo "Set environment variables for build"
export PKG_CONFIG_ALLOW_CROSS=1
export OPENSSL_DIR=/usr
export OPENSSL_LIB_DIR=/usr/lib
export OPENSSL_INCLUDE_DIR=/usr/include
export OPENSSL_STATIC=1
export RUSTFLAGS="-C target-feature=+crt-static -C link-arg=-L/usr/lib -C link-arg=-L/home/QR2M/compile-circus/STATIC/lib/pkgconfig -C link-arg=-lssl -C link-arg=-lcrypto -C link-arg=-static"

echo "Checking pkg-config for dependencies"
for pkg in gtk4 libadwaita-1; do
  echo "Checking $pkg..."
  {
    pkg-config --modversion "$pkg"
  } 2>&1 | tee "$LOG_DIR/qr2m-01-pkgconfig.log"

  STATUS=${PIPESTATUS[0]}
  if [ "$STATUS" -ne 0 ]; then
    cat "$LOG_DIR/qr2m-01-pkgconfig.log"
    echo "ERROR - qr2m - 01/06 - pkgconfig"
    exit 1
  fi
done

echo "Listing .pc files in $STATIC_DIR/lib/pkgconfig:"
ls -l "$STATIC_DIR/lib/pkgconfig"

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

echo "Cloning project..."
{
  git clone https://github.com/control-owl/QR2M.git --depth 1 QR2M
} 2>&1 | tee "$LOG_DIR/qr2m-02-clone.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat "$LOG_DIR/qr2m-02-clone.log"
  echo "ERROR - qr2m - 02/06 - Clone"
  exit 1
fi

cd QR2M

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  cargo build --release --target "$TARGET" --features "$FEATURES" --locked -vv
} 2>&1 | tee "$LOG_DIR/qr2m-03-build.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/qr2m-03-build.log
  echo "ERROR - qr2m - 03/06 - Build"
  exit 1
fi

echo "Listing build directory:"
ls -l "$BUILD_PATH/release"

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  cargo test --release --locked - --no-fail-fast --target "$TARGET" -vv --features "offline" # "$FEATURES"
} 2>&1 | tee "$LOG_DIR/qr2m-04-test.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/qr2m-04-test.log
  echo "ERROR - qr2m - 04/06 - Test"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

echo "Checking binary:"
export BIN="$BUILD_PATH/release/$APP_NAME"
echo "BIN=$BIN"

if [ -f "$BIN" ]; then
  {
    file "$BIN"
  } 2>&1 | tee "$LOG_DIR/qr2m-05-file_check.log"

  STATUS=${PIPESTATUS[0]}
  if [ "$STATUS" -ne 0 ]; then
    cat $LOG_DIR/qr2m-05-file_check.log
    echo "ERROR - qr2m - 05/06 - File check"
  exit 1
  fi

  {
    ldd "$BIN"
  } 2>&1 | tee "$LOG_DIR/qr2m-06-ldd_check.log"

  STATUS=${PIPESTATUS[0]}
  if [ "$STATUS" -ne 0 ]; then
    cat $LOG_DIR/qr2m-06-ldd_check.log
    echo "ERROR - qr2m - 06/06 - Ldd check"
  exit 1
  fi

  chmod +x "$BIN"
else
  echo "Error: Binary not found at $BIN"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

echo "Creating output binary"
if [ "$OUTPUT" = "true" ]; then
  echo "Copying files to $OUTPUT_DIR..."
  mkdir -p "$OUTPUT_DIR"

  if ! cp "$BIN" "$OUTPUT_DIR"; then
    echo "Error: Failed to copy $BIN to $OUTPUT_DIR"
    exit 1
  fi

  #chown 1001:1001 "$OUTPUT_DIR/$APP_NAME" || { echo "Error: Failed to change ownership of $OUTPUT_DIR/$APP_NAME"; exit 1; }

  echo "Listing output directory:"
  ls -l "$OUTPUT_DIR"
fi