#!/bin/bash
set -e

CIRCUS="/home/QR2M/compile-circus"
LOG_DIR="$CIRCUS/LOG"
STATIC_DIR="$CIRCUS/STATIC"

mkdir -p "$CIRCUS"
mkdir -p "$LOG_DIR"
mkdir -p "$STATIC_DIR"

cd "$CIRCUS"

git clone https://github.com/ximion/appstream.git appstream
cd appstream

meson setup builddir \
  --prefix="$STATIC_DIR" \
  --default-library=static \
  -Dstemming=false \
  -Dsystemd=false \
  -Dvapi=false \
  -Dqt=false \
  -Dcompose=false \
  -Dapt-support=false \
  -Dgir=false \
  -Dsvg-support=false \
  -Dzstd-support=false \
  -Ddocs=false \
  -Dapidocs=false \
  -Dinstall-docs=false \
  -Dmaintainer=false \
  -Dstatic-analysis=false 2>&1 | tee "$LOG_DIR/appstream_setup.log"

STATUS=$?
if [ "$STATUS" -ne 0 ]; then
  cat "$LOG_DIR/appstream_setup.log"
  echo "MESON SETUP APPSTREAM FAIL"
  exit 1
fi

meson compile -C builddir 2>&1 | tee "$LOG_DIR/appstream_compile.log"
STATUS=$?
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/appstream_compile.log
  echo "MESON COMPILE APPSTREAM FAIL"
  exit 1
fi

meson install -C builddir 2>&1 | tee "$LOG_DIR/appstream_install.log"
STATUS=$?
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/appstream_install.log
  echo "MESON INSTALL APPSTREAM FAIL"
  exit 1
fi

echo "appstream compiled and installed successfully"