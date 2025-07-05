#!/bin/sh
set -e

CIRCUS="/home/QR2M/compile-circus"
LOG_DIR="$CIRCUS/OUTPUT/"

mkdir -p $CIRCUS
mkdir -p $LOG_DIR

cd $CIRCUS

git clone https://gitlab.gnome.org/GNOME/gtk.git --depth 1 2>&1 | tee "$LOG_DIR/gtk_clone.log"
if [ $? -ne 0 ]; then
  cat $LOG_DIR/gtk_clone.log
  echo "Failed to clone gtk repository"
  exit 1
fi

cd gtk

meson setup builddir \
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
  -Dbuild-tests=false \
  --default-library static 2>&1 | tee "$LOG_DIR/gtk_setup.log"

if [ $? -ne 0 ]; then
  cat $LOG_DIR/gtk_setup.log
  echo "MESON SETUP GTK FAIL"
  exit 1
fi

meson compile -C builddir 2>&1 | tee "$LOG_DIR/gtk_compile.log"
if [ $? -ne 0 ]; then
  cat $LOG_DIR/gtk_compile.log
  echo "MESON COMPILE GTK FAIL"
  exit 1
fi

meson install -C builddir 2>&1 | tee "$LOG_DIR/gtk_install.log"
if [ $? -ne 0 ]; then
  cat $LOG_DIR/gtk_install.log
  echo "MESON INSTALL PANGO FAIL"
  exit 1
fi

echo "gtk compiled and installed successfully"
