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
  git clone https://gitlab.gnome.org/GNOME/gtk.git --depth 1 gtk4
} 2>&1 | tee "$LOG_DIR/gtk4-01-clone.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat "$LOG_DIR/gtk4-01-clone.log"
  echo "ERROR - gtk4 - 01/04 - Clone"
  exit 1
fi

cd gtk4

{
  meson setup builddir \
    --prefix=$STATIC_DIR \
    --default-library static \
    -Dintrospection=disabled \
    -Ddocumentation=false \
    -Dman-pages=false \
    -Dmedia-gstreamer=disabled \
    -Dprint-cpdb=disabled \
    -Dprint-cups=disabled \
    -Dvulkan=disabled \
    -Dcloudproviders=disabled \
    -Dsysprof=disabled \
    -Dtracker=disabled \
    -Dcolord=disabled \
    -Daccesskit=disabled \
    -Dscreenshots=false \
    -Dbuild-demos=false \
    -Dbuild-testsuite=false \
    -Dbuild-examples=false \
  -Dbuild-tests=false
}  2>&1 | tee "$LOG_DIR/gtk4-02-setup.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/gtk4-02-setup.log
  echo "ERROR - gtk4 - 02/04 - Setup"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  ninja -C builddir
} 2>&1 | tee "$LOG_DIR/gtk4-03-compile.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/gtk4-03-compile.log
  echo "ERROR - gtk4 - 03/04 - Compile"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  ninja -C builddir install
} 2>&1 | tee "$LOG_DIR/gtk4-04-install.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/gtk4-04-install.log
  echo "ERROR - gtk4 - 04/04 - Install"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

if [ ! -d "$STATIC_DIR/lib/pkgconfig" ]; then
  mkdir -p "$STATIC_DIR/lib/pkgconfig"
fi

if [ -f "$STATIC_DIR/lib/pkgconfig/gtk4.pc" ]; then
  cp "$STATIC_DIR/lib/pkgconfig/gtk4.pc" "$STATIC_DIR/lib/pkgconfig/gtk-4.pc"
  sed -i "s|Libs:.*|Libs: -L$STATIC_DIR/lib -lgtk-4|" "$STATIC_DIR/lib/pkgconfig/gtk-4.pc"
  sed -i "s|Cflags:.*|Cflags: -I$STATIC_DIR/include/gtk-4.0 -I$STATIC_DIR/include|" "$STATIC_DIR/lib/pkgconfig/gtk-4.pc"
else
  echo "Error: gtk4.pc not found in $STATIC_DIR/lib/pkgconfig"
  if [ -f "$STATIC_DIR/lib/pkgconfig/gtk-4.pc" ]; then
    echo "gtk-4.pc found"
  else
    echo "No gtk*4.pc found"
    exit 1
  fi
fi

if [ -f "$STATIC_DIR/lib/pkgconfig/gtk4.pc" ]; then
  if pkg-config --modversion "$STATIC_DIR/lib/pkgconfig/gtk4.pc" > /dev/null 2>&1; then
    echo "Verified: gtk4.pc is valid"
  else
    echo "Error: pkg-config cannot validate gtk4.pc"
    if [ -f "$STATIC_DIR/lib/pkgconfig/gtk4.pc" ]; then
      cat "$STATIC_DIR/lib/pkgconfig/gtk4.pc"
    fi
    exit 1
  fi
fi

if [ -f "$STATIC_DIR/lib/libgtk-4.so" ]; then
  ar rcs "$STATIC_DIR/lib/libgtk-4.a" "$STATIC_DIR/lib/libgtk-4.so"
  if [ $? -ne 0 ]; then
    echo "Error: Failed to create libgtk-4.a from libgtk-4.so"
    exit 1
  fi
else
  echo "Error: Shared library $STATIC_DIR/lib/libgtk-4.so not found"
  if [ -d "$STATIC_DIR/lib" ]; then
    ls -l "$STATIC_DIR/lib/libgtk*"
  fi
  exit 1
fi

if [ ! -f "$STATIC_DIR/lib/libgtk-4.a" ]; then
  echo "Error: Library $STATIC_DIR/lib/libgtk-4.a not found after creation"
  if [ -d "$STATIC_DIR/lib" ]; then
    ls -l "$STATIC_DIR/lib/libgtk*.a"
  fi
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

echo "gtk compiled and installed successfully"

