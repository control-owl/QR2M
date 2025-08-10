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

export PATH="/home/QR2M/compile-circus/STATIC/bin:$PATH"
export PKG_CONFIG_PATH="/home/QR2M/compile-circus/STATIC/lib/pkgconfig:/home/QR2M/compile-circus/STATIC/lib64/pkgconfig:/home/QR2M/compile-circus/STATIC/share/pkgconfig"
export CFLAGS="-I$STATIC_DIR/include -O2 -fno-semantic-interposition -fPIC"
export LDFLAGS="-L$STATIC_DIR/lib -L$STATIC_DIR/lib64 -L/usr/lib/gcc/x86_64-alpine-linux-musl/14.2.0 -static"
export RUSTFLAGS="-C target-feature=+crt-static -C link-arg=-static -C link-arg=-L$STATIC_DIR/lib -C link-arg=-L$STATIC_DIR/lib64 -C link-arg=-L$STATIC_DIR/share -C link-arg=-lappstream -C link-arg=-ladwaita-1 -C link-arg=-lgtk-4 -C link-arg=-lgdk_pixbuf-2.0 -C link-arg=-lcairo -C link-arg=-lpango-1.0 -C link-arg=-lpangocairo-1.0 -C link-arg=-lharfbuzz -C link-arg=-lharfbuzz-subset -C link-arg=-lfribidi -C link-arg=-lfontconfig -C link-arg=-lfreetype -C link-arg=-lxml2 -C link-arg=-lz -C link-arg=-lpng16 -C link-arg=-ltiff -C link-arg=-ljpeg -C link-arg=-lxkbcommon -C link-arg=-lX11 -C link-arg=-lXext -C link-arg=-lXrender -C link-arg=-lXrandr -C link-arg=-lXi -C link-arg=-lXfixes -C link-arg=-lXcursor -C link-arg=-lXdamage -C link-arg=-lXinerama -C link-arg=-lXcomposite -C link-arg=-lxcb -C link-arg=-lxcb-render -C link-arg=-lxcb-shm -C link-arg=-lXau -C link-arg=-lXdmcp -C link-arg=-lpixman-1 -C link-arg=-lglib-2.0 -C link-arg=-lgobject-2.0 -C link-arg=-lgio-2.0 -C link-arg=-lgmodule-2.0 -C link-arg=-lffi -C link-arg=-lpcre2-8 -C link-arg=-lepoxy -C link-arg=-lgraphene-1.0 -C link-arg=-lssl -C link-arg=-lcrypto -C link-arg=-lcurl -C link-arg=-lnghttp2 -C link-arg=-lbz2 -C link-arg=-lbrotlidec -C link-arg=-lbrotlicommon -C link-arg=-llzma -C link-arg=-lunistring -C link-arg=-lyaml -C link-arg=-leconf -C link-arg=-latomic -C link-arg=-lstdc++ -C link-arg=-lm -C link-arg=-lintl -C link-arg=-lexpat -C link-arg=-ldrm -C link-arg=-lsass"

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  git clone --branch docker --depth 1 https://github.com/control-owl/QR2M.git QR2M
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
  cargo test --release --locked --no-fail-fast --target "$TARGET" -vv --features "$FEATURES"
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
    echo "Checking binary with file:"
    file "$BIN"
  } 2>&1 | tee -a "$LOG_FILE"

  STATUS=${PIPESTATUS[0]}
  if [ "$STATUS" -ne 0 ]; then
    cat "$LOG_FILE"
  exit 1
  fi

  {
    echo "Checking binary with ldd:"
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