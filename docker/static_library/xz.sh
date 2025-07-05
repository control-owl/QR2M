#!/bin/bash
set -e

CIRCUS="/home/QR2M/compile-circus"
LOG_DIR="$CIRCUS/OUTPUT/"

mkdir -p $CIRCUS
mkdir -p $LOG_DIR

cd $CIRCUS

git clone https://github.com/tukaani-project/xz.git xz 2>&1 | tee "$LOG_DIR/xz_clone.log"
STATUS=$?
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/xz_clone.log
  echo "Failed to clone xz repository"
  exit 1
fi

cd xz

./autogen.sh 2>&1 | tee "$LOG_DIR/xz_autogen.log"
STATUS=$?
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/xz_autogen.log
  echo "autogen.sh failed for xz"
  exit 1
fi

./configure --enable-static --disable-shared --prefix=/usr/local 2>&1 | tee "$LOG_DIR/xz_configure.log"
STATUS=$?
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/xz_configure.log
  echo "configure failed for xz"
  exit 1
fi

make -j"$(nproc)" 2>&1 | tee "$LOG_DIR/xz_make.log"
STATUS=$?
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/xz_make.log
  echo "make failed for xz"
  exit 1
fi

make install 2>&1 | tee "$LOG_DIR/xz_install.log"
STATUS=$?
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/xz_install.log
  echo "make install failed for xz"
  exit 1
fi

echo "xz compiled and installed successfully"
