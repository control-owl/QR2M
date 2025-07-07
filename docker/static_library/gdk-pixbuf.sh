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
  git clone https://gitlab.gnome.org/GNOME/gdk-pixbuf.git pixbuf
} 2>&1 | tee "$LOG_DIR/pixbuf-01-clone.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat "$LOG_DIR/pixbuf-01-clone.log"
  echo "ERROR - pixbuf - 01/04 - Clone"
  exit 1
fi

cd pixbuf

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  meson setup builddir \
    --default-library=static \
    --prefix=$STATIC_DIR \
    -Dpng=enabled \
    -Dtiff=disabled \
    -Djpeg=disabled \
    -Dgif=enabled \
    -Dglycin=disabled \
    -Dandroid=disabled \
    -Dothers=disabled \
    -Ddocumentation=false \
    -Dintrospection=disabled \
    -Dman=false \
    -Drelocatable=false \
    -Dnative_windows_loaders=false \
    -Dtests=false \
    -Dinstalled_tests=false \
    -Dgio_sniffing=false \
    -Dthumbnailer=disabled 
} 2>&1 | tee "$LOG_DIR/pixbuf-02-setup.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/pixbuf-02-setup.log
  echo "ERROR - pixbuf - 02/04 - Setup"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  meson compile -C builddir
} 2>&1 | tee "$LOG_DIR/pixbuf-03-compile.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/pixbuf-03-compile.log
  echo "ERROR - pixbuf - 03/04 - Compile"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{ 
  meson install -C builddir
} 2>&1 | tee "$LOG_DIR/pixbuf-04-install.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/pixbuf-04-install.log
  echo "ERROR - pixbuf - 04/04 - Install"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

echo "pixbuf compiled and installed successfully"
