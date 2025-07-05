#!/bin/bash
set -e

CIRCUS="/home/QR2M/compile-circus"
LOG_DIR="$CIRCUS/LOG"
STATIC_DIR="$CIRCUS/STATIC"

mkdir -p $CIRCUS
mkdir -p $LOG_DIR
mkdir -p $STATIC_DIR

cd $CIRCUS

git clone https://gitlab.gnome.org/GNOME/librsvg.git librsvg
cd librsvg

meson setup builddir \
  --prefix=/usr/local \
  --buildtype=release \
  --prefix=$STATIC_DIR \
  -Ddefault_library=static \
  -Ddocs=false \
  -Dtests=false \
  -Dvala=false 2>&1 | tee "$LOG_DIR/librsvg_setup.log"
STATUS=$?
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/librsvg_setup.log
  echo "MESON SETUP LIBRSVG FAIL"
  exit 1
fi

ninja -C builddir 2>&1 | tee "$LOG_DIR/librsvg_ninja.log"
STATUS=$?
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/librsvg_ninja.log
  echo "NINJA COMPILE LIBRSVG FAIL"
  exit 1
fi

echo "Installing librsvg..."
ninja -C builddir install 2>&1 | tee "$LOG_DIR/librsvg_install.log"
STATUS=$?
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/librsvg_install.log
  echo "NINJA INSTALL LIBRSVG FAIL"
  exit 1
fi

echo "librsvg compiled and installed successfully"
