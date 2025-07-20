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
  git clone https://gitlab.gnome.org/GNOME/libadwaita.git --depth 1 libadwaita
} 2>&1 | tee "$LOG_DIR/libadwaita-01-clone.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat "$LOG_DIR/libadwaita-01-clone.log"
  echo "ERROR - libadwaita - 01/04 - Clone"
  exit 1
fi

cd libadwaita

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  export PKG_CONFIG_PATH="$STATIC_DIR/lib/pkgconfig"
  export PKG_CONFIG="pkg-config --static"
  export RUSTFLAGS="-C link-arg=-L$STATIC_DIR/lib -C link-arg=-lz -C link-arg=-latomic"
  export CFLAGS="-I$STATIC_DIR/include -O2 -fno-semantic-interposition -Wno-maybe-uninitialized"
  export LDFLAGS="-L$STATIC_DIR/lib -lz -latomic"
  PKG_CONFIG_LIBDIR="$STATIC_DIR/lib/pkgconfig" meson setup builddir \
    --default-library static \
    --prefix=$STATIC_DIR \
    -Dexamples=false \
    -Dgtk_doc=false \
    -Ddocumentation=false \
    -Dintrospection=disabled \
    -Dvapi=false \
    -Dtests=false
}  2>&1 | tee "$LOG_DIR/libadwaita-02-setup.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/libadwaita-02-setup.log
  echo "ERROR - libadwaita - 02/04 - Setup"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  ninja -C builddir 2>&1
} | tee "$LOG_DIR/libadwaita-03-compile.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/libadwaita-03-compile.log
  echo "ERROR - libadwaita - 03/04 - Compile"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  ninja -C builddir install
} 2>&1 | tee "$LOG_DIR/libadwaita-04-install.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/libadwaita-04-install.log
  echo "ERROR - libadwaita - 01/04 - Install"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

echo "libadwaita compiled and installed successfully"
