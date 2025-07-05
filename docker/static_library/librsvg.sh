#!/bin/sh
set -e

CIRCUS="/home/QR2M/compile-circus"
LOG_DIR="$CIRCUS/OUTPUT/"

mkdir -p $CIRCUS
mkdir -p $LOG_DIR

cd $CIRCUS

git clone https://gitlab.gnome.org/GNOME/librsvg.git 2>&1 | tee "$LOG_DIR/librsvg_clone.log"
if [ $? -ne 0 ]; then
  cat $LOG_DIR/librsvg_clone.log
  echo "Failed to clone librsvg repository"
  exit 1
fi

cd librsvg

meson setup builddir \
  --prefix=/usr/local \
  --buildtype=release \
  -Ddefault_library=static \
  -Ddocs=false \
  -Dtests=false \
  -Dvala=false 2>&1 | tee "$LOG_DIR/librsvg_setup.log"

if [ $? -ne 0 ]; then
  cat $LOG_DIR/librsvg_setup.log
  echo "MESON SETUP LIBRSVG FAIL"
  exit 1
fi

ninja -C builddir 2>&1 | tee "$LOG_DIR/librsvg_ninja.log"

if [ $? -ne 0 ]; then
  cat $LOG_DIR/librsvg_ninja.log
  echo "NINJA COMPILE LIBRSVG FAIL"
  exit 1
fi

echo "Installing librsvg..."
ninja -C builddir install 2>&1 | tee "$LOG_DIR/librsvg_install.log"

if [ $? -ne 0 ]; then
  cat $LOG_DIR/librsvg_install.log
  echo "NINJA INSTALL LIBRSVG FAIL"
  exit 1
fi

echo "librsvg compiled and installed successfully"
