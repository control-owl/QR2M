#!/bin/sh
set -e

CIRCUS="/home/QR2M/compile-circus"
LOG_DIR="$CIRCUS/OUTPUT/"

mkdir -p $CIRCUS
mkdir -p $LOG_DIR

cd $CIRCUS

git clone https://gitlab.gnome.org/GNOME/libadwaita.git --depth 1 2>&1 | tee "$LOG_DIR/libadwaita_clone.log"
if [ $? -ne 0 ]; then
  cat $LOG_DIR/libadwaita_clone.log
  echo "CLONE ADWAITA FAIL"
  exit 1
fi

cd libadwaita

meson setup builddir \
  -Dexamples=false \
  -Dtests=false \
  --default-library static 2>&1 | tee "$LOG_DIR/libadwaita_setup.log"

if [ $? -ne 0 ]; then
  cat $LOG_DIR/libadwaita_setup.log
  echo "MESON SETUP ADWAITA FAIL"
  exit 1
fi

ninja -C builddir 2>&1 | tee "$LOG_DIR/libadwaita_ninja.log"
if [ $? -ne 0 ]; then
  cat $LOG_DIR/libadwaita_ninja.log
  echo "NINJA COMPILE ADWAITA FAIL"
  exit 1
fi

ninja -C builddir install 2>&1 | tee "$LOG_DIR/libadwaita_install.log"
if [ $? -ne 0 ]; then
  cat $LOG_DIR/libadwaita_install.log
  echo "NINJA INSTALL ADWAITA FAIL"
  exit 1
fi

echo "libadwaita compiled and installed successfully"
