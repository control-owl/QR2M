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

export PKG_CONFIG_LIBDIR="/home/QR2M/compile-circus/STATIC/lib/pkgconfig"
export PKG_CONFIG_PATH="/home/QR2M/compile-circus/STATIC/share/pkgconfig:/usr/lib/pkgconfig:/usr/share/pkgconfig:/usr/lib/x86_64-linux-musl/pkgconfig:/usr/local/lib/pkgconfig"
export PKG_CONFIG="pkg-config --static"
export CFLAGS="-I/home/QR2M/compile-circus/STATIC/include -O2 -fno-semantic-interposition -Wno-maybe-uninitialized"
export LDFLAGS="-L/home/QR2M/compile-circus/STATIC/lib -lz -latomic"
export RUSTFLAGS="-C link-arg=-L/home/QR2M/compile-circus/STATIC/lib -C link-arg=-lz -C link-arg=-latomic"

cd "$CIRCUS"

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  git clone https://github.com/fribidi/fribidi.git --depth 1 fribidi
} 2>&1 | tee "$LOG_DIR/fribidi-01-clone.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat "$LOG_DIR/fribidi-01-clone.log"
  exit 1
fi

cd fribidi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  meson setup builddir \
    -Dprefix="$STATIC_DIR" \
    -Ddefault_library=static \
    -Ddeprecated=false \
    -Ddocs=false \
    -Dbin=false \
    -Dtests=false \
    -Dfuzzer_ldflags='' \
    -Dbuildtype=release \
    -Dc_args='-O2 -fno-semantic-interposition' \
    -Dpkg_config_path="$STATIC_DIR/lib/pkgconfig"
} 2>&1 | tee "$LOG_DIR/fribidi-02-setup.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/fribidi-02-setup.log
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  ninja -C builddir
} 2>&1 | tee "$LOG_DIR/fribidi-03-compile.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/fribidi-03-compile.log
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  ninja -C builddir install
} 2>&1 | tee "$LOG_DIR/fribidi-04-install.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/fribidi-04-install.log
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

echo "fribidi compiled and installed successfully"