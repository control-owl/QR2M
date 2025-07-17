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
  git clone https://github.com/harfbuzz/harfbuzz.git harfbuzz
} 2>&1 | tee "$LOG_DIR/harfbuzz-01-clone.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat "$LOG_DIR/harfbuzz-01-clone.log"
  echo "ERROR - harfbuzz - 01/04 - Clone"
  exit 1
fi

cd harfbuzz

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  meson setup builddir \
    -Dprefix="$STATIC_DIR" \
    -Ddefault_library=static \
    -Dfreetype=enabled \
    -Dglib=enabled \
    -Dcairo=enabled \
    -Dtests=disabled \
    -Ddocs=disabled \
    --buildtype=release
} 2>&1 | tee "$LOG_DIR/harfbuzz-02-setup.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/harfbuzz-02-setup.log
  echo "ERROR - harfbuzz - 02/04 - Setup"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  ninja -C builddir
} 2>&1 | tee "$LOG_DIR/harfbuzz-03-compile.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/harfbuzz-03-compile.log
  echo "ERROR - harfbuzz - 03/04 - Compile"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  ninja -C builddir install
} 2>&1 | tee "$LOG_DIR/harfbuzz-04-install.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/harfbuzz-04-install.log
  echo "ERROR - harfbuzz - 04/04 - Install"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

echo "harfbuzz compiled and installed successfully"