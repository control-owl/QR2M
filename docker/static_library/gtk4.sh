#!/bin/bash
set -e

CIRCUS="/home/QR2M/compile-circus"
LOG_DIR="$CIRCUS/LOG"
STATIC_DIR="$CIRCUS/STATIC"

mkdir -p $CIRCUS
mkdir -p $LOG_DIR
mkdir -p $STATIC_DIR

cd $CIRCUS

git clone https://gitlab.gnome.org/GNOME/gtk.git --depth 1 gtk
cd gtk

meson setup builddir \
  --default-library static \
  --prefix=$STATIC_DIR \
  -Dmedia-gstreamer=disabled \
  -Dprint-cpdb=disabled \
  -Dprint-cups=disabled \
  -Dvulkan=disabled \
  -Dcloudproviders=disabled \
  -Dsysprof=disabled \
  -Dtracker=disabled \
  -Dcolord=disabled \
  -Daccesskit=disabled \
  -Dintrospection=disabled \
  -Ddocumentation=false \
  -Dscreenshots=false \
  -Dman-pages=false \
  -Dbuild-demos=false \
  -Dbuild-testsuite=false \
  -Dbuild-examples=false \
  -Dbuild-tests=false  2>&1 | tee "$LOG_DIR/gtk_setup.log"
STATUS=$?
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/gtk_setup.log
  echo "MESON SETUP GTK FAIL"
  exit 1
fi

meson compile -C builddir 2>&1 | tee "$LOG_DIR/gtk_compile.log"
STATUS=$?
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/gtk_compile.log
  echo "MESON COMPILE GTK FAIL"
  exit 1
fi

meson install -C builddir 2>&1 | tee "$LOG_DIR/gtk_install.log"
STATUS=$?
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/gtk_install.log
  echo "MESON INSTALL GTK FAIL"
  exit 1
fi


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

echo "gtk compiled and installed successfully"

