#!/bin/bash
set -e

CIRCUS="/home/QR2M/compile-circus"
LOG_DIR="$CIRCUS/LOG"
STATIC_DIR="$CIRCUS/STATIC"

mkdir -p $CIRCUS
mkdir -p $LOG_DIR
mkdir -p $STATIC_DIR

cd $CIRCUS

git clone https://github.com/tukaani-project/xz.git xz
cd xz

./autogen.sh 2>&1 | tee "$LOG_DIR/xz_autogen.log"
STATUS=$?
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/xz_autogen.log
  echo "autogen.sh failed for xz"
  exit 1
fi

./configure \
  --enable-static \
  --disable-shared \
  --prefix=$STATIC_DIR 2>&1 | tee "$LOG_DIR/xz_configure.log"

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
