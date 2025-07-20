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

export PKG_CONFIG_LIBDIR="/home/QR2M/compile-circus/STATIC/lib/pkgconfig"
export PKG_CONFIG_PATH="/home/QR2M/compile-circus/STATIC/share/pkgconfig"
export PKG_CONFIG="pkg-config --static"
export CFLAGS="-I/home/QR2M/compile-circus/STATIC/include -O2 -fno-semantic-interposition -Wno-maybe-uninitialized"
export LDFLAGS="-L/home/QR2M/compile-circus/STATIC/lib -lz -latomic"
export RUSTFLAGS="-C link-arg=-L/home/QR2M/compile-circus/STATIC/lib -C link-arg=-lz -C link-arg=-latomic"

cd "$CIRCUS"

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

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