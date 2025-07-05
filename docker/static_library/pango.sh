#!/bin/bash
set -e

CIRCUS="/home/QR2M/compile-circus"
LOG_DIR="$CIRCUS/OUTPUT/"

mkdir -p $CIRCUS
mkdir -p $LOG_DIR

cd $CIRCUS

git clone https://github.com/GNOME/pango.git pango 2>&1 | tee "$LOG_DIR/pango_clone.log"
STATUS=$?
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/pango_clone.log
  echo "Failed to clone pango repository"
  exit 1
fi

cd pango

meson setup builddir \
  -Ddocumentation=false \
  -Dgtk_doc=false \
  -Dman-pages=false \
  -Dintrospection=disabled \
  -Dbuild-testsuite=false \
  -Dbuild-examples=false \
  -Dsysprof=disabled \
  -Dlibthai=disabled \
  -Dxft=disabled \
  --default-library static 2>&1 | tee "$LOG_DIR/pango_setup.log"
STATUS=$?
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/pango_setup.log
  echo "MESON SETUP PANGO FAIL"
  exit 1
fi

meson compile -C builddir 2>&1 | tee "$LOG_DIR/pango_compile.log"
STATUS=$?
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/pango_compile.log
  echo "MESON COMPILE PANGO FAIL"
  exit 1
fi

meson install -C builddir 2>&1 | tee "$LOG_DIR/pango_install.log"
STATUS=$?
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/pango_install.log
  echo "MESON INSTALL PANGO FAIL"
  exit 1
fi
