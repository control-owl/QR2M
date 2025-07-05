#!/bin/bash
set -e

CIRCUS="/home/QR2M/compile-circus"
LOG_DIR="$CIRCUS/OUTPUT/"

mkdir -p $CIRCUS
mkdir -p $LOG_DIR

cd $CIRCUS

git clone https://gitlab.freedesktop.org/xorg/lib/libxau.git libxau 2>&1 | tee "$LOG_DIR/libxau_clone.log"
STATUS=$?
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/libxau_clone.log
  echo "Failed to clone libXau repository"
  exit 1
fi

cd libxau

./autogen.sh --enable-static --disable-shared --prefix=/usr/local 2>&1 | tee "$LOG_DIR/libxau_autogen.log"
STATUS=$?
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/libxau_autogen.log
  echo "autogen.sh failed for libXau"
  exit 1
fi

make -j"$(nproc)" 2>&1 | tee "$LOG_DIR/libxau_make.log"
STATUS=$?
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/libxau_make.log
  echo "make failed for libXau"
  exit 1
fi

make install 2>&1 | tee "$LOG_DIR/libxau_install.log"
STATUS=$?
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/libxau_install.log
  echo "make install failed for libXau"
  exit 1
fi

echo "libXau compiled and installed successfully"

