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
LOG_FILE="$LOG_DIR/$(basename "$0").log"
STATIC_DIR="$CIRCUS/STATIC"

mkdir -p "$CIRCUS"
mkdir -p "$LOG_DIR"
mkdir -p "$STATIC_DIR"

cd "$CIRCUS"

export PKG_CONFIG_LIBDIR="/home/QR2M/compile-circus/STATIC/lib/pkgconfig"
export PKG_CONFIG_PATH="/home/QR2M/compile-circus/STATIC/share/pkgconfig:/usr/lib/pkgconfig:/usr/share/pkgconfig:/usr/lib/x86_64-linux-musl/pkgconfig:/usr/local/lib/pkgconfig"
#export PKG_CONFIG="pkg-config --static"
export CFLAGS="-I/home/QR2M/compile-circus/STATIC/include -O2 -fno-semantic-interposition -Wno-maybe-uninitialized"
export LDFLAGS="-L/home/QR2M/compile-circus/STATIC/lib -L/home/QR2M/compile-circus/STATIC/lib64"
export RUSTFLAGS="-C link-arg=-L/home/QR2M/compile-circus/STATIC/lib -C link-arg=-L/home/QR2M/compile-circus/STATIC/lib64"

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  git clone https://github.com/control-owl/QR2M.git --depth 1 QR2M
} 2>&1 | tee -a "$LOG_FILE"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat "$LOG_FILE"
  exit 1
fi

cd QR2M

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  cargo build --release --target "$TARGET" --features "$FEATURES" --locked -vv
} 2>&1 | tee -a "$LOG_FILE"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat "$LOG_FILE"
  exit 1
fi

echo "Listing build directory:"
ls -l "$BUILD_PATH/release"

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  cargo test --release --locked - --no-fail-fast --target "$TARGET" -vv --features "offline" # "$FEATURES"
} 2>&1 | tee -a "$LOG_FILE"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat "$LOG_FILE"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

echo "Checking binary:"
export BIN="$BUILD_PATH/release/$APP_NAME"
echo "BIN=$BIN"

if [ -f "$BIN" ]; then
  {
    file "$BIN"
  } 2>&1 | tee -a "$LOG_FILE"

  STATUS=${PIPESTATUS[0]}
  if [ "$STATUS" -ne 0 ]; then
    cat "$LOG_FILE"
  exit 1
  fi

  {
    ldd "$BIN"
  } 2>&1 | tee -a "$LOG_FILE"

  STATUS=${PIPESTATUS[0]}
  if [ "$STATUS" -ne 0 ]; then
    cat "$LOG_FILE"
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