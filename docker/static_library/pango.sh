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
  git clone https://github.com/GNOME/pango.git --depth 1 pango
} 2>&1 | tee "$LOG_DIR/pango-01-clone.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat "$LOG_DIR/pango-01-clone.log"
  echo "ERROR - pango - 01/04 - Clone"
  exit 1
fi

cd pango

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  export PKG_CONFIG_PATH="$STATIC_DIR/lib/pkgconfig"
  export CFLAGS="-O2 -fno-semantic-interposition -Wno-maybe-uninitialized"
  export LDFLAGS="-L$STATIC_DIR/lib -latomic"
  PKG_CONFIG_LIBDIR="$STATIC_DIR/lib/pkgconfig" meson setup builddir \
    --default-library static \
    --prefix=$STATIC_DIR \
    -Ddocumentation=false \
    -Dman-pages=false \
    -Dintrospection=disabled \
    -Dbuild-testsuite=false \
    -Dbuild-examples=false \
    -Dsysprof=disabled \
    -Dlibthai=disabled \
    -Dfreetype=disabled \
    -Dxft=disabled
} 2>&1 | tee "$LOG_DIR/pango-02-setup.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/pango-02-setup.log
  echo "ERROR - pango - 02/04 - Setup"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  ninja -C builddir
} 2>&1 | tee "$LOG_DIR/pango-03-compile.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/pango-03-compile.log
  echo "ERROR - pango - 03/04 - Compile"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  ninja -C builddir install
} 2>&1 | tee "$LOG_DIR/pango-04-install.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/pango-04-install.log
  echo "ERROR - pango - 04/04 - Install"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

echo "pango compiled and installed successfully"