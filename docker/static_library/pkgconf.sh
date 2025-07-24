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
LOG_FILE="$LOG_DIR/$(basename "$0").log"
STATIC_DIR="$CIRCUS/STATIC"

mkdir -p "$CIRCUS"
mkdir -p "$LOG_DIR"
mkdir -p "$STATIC_DIR"

cd "$CIRCUS"

export PKG_CONFIG_LIBDIR="/home/QR2M/compile-circus/STATIC/lib/pkgconfig"
export PKG_CONFIG_PATH="/home/QR2M/compile-circus/STATIC/share/pkgconfig:/usr/lib/pkgconfig:/usr/share/pkgconfig:/usr/lib/x86_64-linux-musl/pkgconfig:/usr/local/lib/pkgconfig"
export PKG_CONFIG="pkg-config --static"
export CFLAGS="-I/home/QR2M/compile-circus/STATIC/include -O2 -fno-semantic-interposition -Wno-maybe-uninitialized"
export LDFLAGS="-L/home/QR2M/compile-circus/STATIC/lib"
export PATH="/home/QR2M/compile-circus/STATIC/bin:$PATH"

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  needed_files=()

  source "$PROJECT_DIR/check_me_baby.sh" "${needed_files[@]}"
} 2>&1 | tee "$LOG_FILE"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat "$LOG_FILE"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  git clone https://github.com/pkgconf/pkgconf.git --depth 1 pkgconf
} 2>&1 | tee "$LOG_FILE"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat "$LOG_FILE"
  exit 1
fi

cd pkgconf

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  ./autogen.sh
} 2>&1 | tee "$LOG_FILE"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat "$LOG_FILE"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  meson setup builddir \
    --prefix=$STATIC_DIR \
    -Ddefault_library=static \
    -Dtests=disabled \
    -Dwith-system-libdir=/home/QR2M/compile-circus/STATIC/lib \
    -Dwith-system-includedir=/home/QR2M/compile-circus/STATIC/include
} 2>&1 | tee "$LOG_FILE"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat "$LOG_FILE"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  ninja -C builddir
} 2>&1 | tee "$LOG_FILE"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat "$LOG_FILE"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  ninja -C builddir install
} 2>&1 | tee "$LOG_FILE"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat "$LOG_FILE"
  exit 1
fi

# ln -sf /home/QR2M/compile-circus/STATIC/bin/pkgconf /usr/bin/pkg-config
ln -sf /home/QR2M/compile-circus/STATIC/bin/pkgconf /home/QR2M/compile-circus/STATIC/bin/pkg-config

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  compiled_files=(
    "libpkgconf.a"
    "libpkgconf.pc"
    "pkgconf"
    "bomtool"
  )

  source "$PROJECT_DIR/check_me_baby.sh" "${compiled_files[@]}"
} 2>&1 | tee "$LOG_FILE"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat "$LOG_FILE"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

echo "pkgconf compiled and installed successfully"
