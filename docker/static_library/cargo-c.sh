#!/bin/bash
# authors = ["Control Owl <qr2m[at]r-o0-t[dot]wtf>"]
# license = "CC-BY-NC-ND-4.0  [2023-2025]  Control Owl"
#
# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

set -e
set -x
set -o pipefail

CIRCUS="/home/QR2M/compile-circus"
LOG_DIR="$CIRCUS/LOG"
STATIC_DIR="$CIRCUS/STATIC"

mkdir -p "$CIRCUS"
mkdir -p "$LOG_DIR"
mkdir -p "$STATIC_DIR"

cd "$CIRCUS"

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  export PKG_CONFIG_PATH="$STATIC_DIR/lib/pkgconfig"
  export PKG_CONFIG="pkg-config --static"
  export ZLIB_STATIC=1
  export PKG_CONFIG_zlib_STATIC="true"
  export RUSTFLAGS="-C link-arg=-L$STATIC_DIR/lib -C link-arg=-lz -C link-arg=-latomic"
  export CFLAGS="-I$STATIC_DIR/include -O2 -fno-semantic-interposition -Wno-maybe-uninitialized"
  export LDFLAGS="-L$STATIC_DIR/lib -lz -latomic"
  cargo install cargo-c --features=vendored-openssl --root "$STATIC_DIR" --locked --verbose
} 2>&1 | tee "$LOG_DIR/cargo-c-01-install.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/cargo-c-01-install.log
  echo "ERROR - cargo-c - 01/02 - Install"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  cargo-cbuild --version
} 2>&1 | tee "$LOG_DIR/cargo-c-02-verify.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/cargo-c-01-install.log
  echo "ERROR - cargo-c - 02/02 - Verify"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

echo "cargo-c installed successfully"