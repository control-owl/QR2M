#!/bin/bash
set -e

CIRCUS="/home/QR2M/compile-circus"
LOG_DIR="$CIRCUS/LOG"
STATIC_DIR="$CIRCUS/STATIC"

mkdir -p $CIRCUS
mkdir -p $LOG_DIR
mkdir -p $STATIC_DIR

cd $CIRCUS

git clone https://gitlab.gnome.org/GNOME/libadwaita.git --depth 1 libadwaita
cd libadwaita

meson setup builddir \
  --default-library static \
  --prefix=$STATIC_DIR \
  -Dexamples=false \
  -Dtests=false  2>&1 | tee "$LOG_DIR/libadwaita_setup.log"
STATUS=$?
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/libadwaita_setup.log
  echo "MESON SETUP ADWAITA FAIL"
  exit 1
fi

ninja -C builddir 2>&1 | tee "$LOG_DIR/libadwaita_ninja.log"
STATUS=$?
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/libadwaita_ninja.log
  echo "NINJA COMPILE ADWAITA FAIL"
  exit 1
fi

ninja -C builddir install 2>&1 | tee "$LOG_DIR/libadwaita_install.log"
STATUS=$?
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/libadwaita_install.log
  echo "NINJA INSTALL ADWAITA FAIL"
  exit 1
fi

echo "libadwaita compiled and installed successfully"
