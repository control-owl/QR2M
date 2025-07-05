#!/bin/bash
set -e

CIRCUS="/home/QR2M/compile-circus"
LOG_DIR="$CIRCUS/OUTPUT/"

mkdir -p $CIRCUS
mkdir -p $LOG_DIR

cd $CIRCUS

git clone https://gitlab.gnome.org/GNOME/gdk-pixbuf.git pixbuf 2>&1 | tee "$LOG_DIR/pixbuf_clone.log"
STATUS=$?
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/pixbuf_clone.log
  echo "Failed to clone pixbuf repository"
  exit 1
fi

cd pixbuf

meson setup builddir \
  --default-library=static \
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
  -Dthumbnailer=disabled 2>&1 | tee "$LOG_DIR/pixbuf_setup.log"

STATUS=$?
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/pixbuf_setup.log
  echo "Meson setup failed for pixbuf"
  exit 1
fi

meson compile -C builddir 2>&1 | tee "$LOG_DIR/pixbuf_compile.log"
STATUS=$?
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/pixbuf_compile.log
  echo "Meson compile failed for pixbuf"
  exit 1
fi

meson install -C builddir 2>&1 | tee "$LOG_DIR/pixbuf_install.log"
STATUS=$?
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/pixbuf_install.log
  echo "Meson install failed for pixbuf"
  exit 1
fi

echo "pixbuf compiled and installed successfully"
