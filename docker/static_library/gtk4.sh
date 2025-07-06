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
  echo "MESON INSTALL PANGO FAIL"
  exit 1
fi

cp $STATIC_DIR/lib/gtk4.pc /usr/lib/pkgconfig/gtk-4.pc

echo "gtk compiled and installed successfully"
