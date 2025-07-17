#!/bin/bash
# authors = ["Control Owl <qr2m[at]r-o0-t[dot]wtf>"]
# license = "CC-BY-NC-ND-4.0  [2023-2025]  Control Owl"
#
# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

set -e
set -x
set -o pipefail

CIRCUS="/home/QR2M/compile-circus"
LOG_DIR="$CIRCUS/LOG"
STATIC_DIR="$CIRCUS/STATIC"

mkdir -p "$CIRCUS"
mkdir -p "$LOG_DIR"
mkdir -p "$STATIC_DIR"

cd "$CIRCUS"

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  git clone https://github.com/libjpeg-turbo/libjpeg-turbo.git libjpeg-turbo
} 2>&1 | tee "$LOG_DIR/libjpeg-turbo-01-clone.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat "$LOG_DIR/libjpeg-turbo-01-clone.log"
  echo "ERROR - libjpeg-turbo - 01/05 - Clone"
  exit 1
fi

cd libjpeg-turbo

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

# {
#   ./autogen.sh
# } 2>&1 | tee "$LOG_DIR/libjpeg-turbo-02-autogen.log"
# 
# STATUS=${PIPESTATUS[0]}
# if [ "$STATUS" -ne 0 ]; then
#   cat $LOG_DIR/libjpeg-turbo-02-autogen.log
#   echo "ERROR - libjpeg-turbo - 02/05 - Clone"
#   exit 1
# fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

mkdir -p builddir
cd builddir

{
  cmake -DCMAKE_INSTALL_PREFIX=$STATIC_DIR -DBUILD_SHARED_LIBS=OFF ..
} 2>&1 | tee "$LOG_DIR/libjpeg-turbo-03-configure.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/libjpeg-turbo-03-configure.log
  echo "ERROR - libjpeg-turbo - 03/05 - Configure"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  make -j"$(nproc)"
} 2>&1 | tee "$LOG_DIR/libjpeg-turbo-04-make.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/libjpeg-turbo-04-make.log
  echo "ERROR - libjpeg-turbo - 04/05 - Compile"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  make install
} 2>&1 | tee "$LOG_DIR/libjpeg-turbo-05-install.log"
STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/libjpeg-turbo-05-install.log
  echo "ERROR - libjpeg-turbo - 05/05 - Install"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

echo "libjpeg-turbo compiled and installed successfully"
