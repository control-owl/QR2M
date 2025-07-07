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
  git clone https://github.com/curl/curl curl
} 2>&1 | tee "$LOG_DIR/curl-01-clone.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat "$LOG_DIR/curl-01-clone.log"
  echo "ERROR - curl - 01/04 - Clone"
  exit 1
fi

cd curl

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  ./configure \
  --enable-static \
  --prefix=$STATIC_DIR \
   --with-ares 
} 2>&1 | tee "$LOG_DIR/curl-02-configure.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/curl-02-configure.log
  echo "ERROR - curl - 02/04 - Clone"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  make -j"$(nproc)"
} 2>&1 | tee "$LOG_DIR/curl-03-make.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/curl-03-make.log
  echo "ERROR - curl - 03/04 - Clone"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  make install
} 2>&1 | tee "$LOG_DIR/curl-04-install.log"
STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/curl-04-install.log
  echo "ERROR - curl - 04/04 - Clone"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

echo "curl compiled and installed successfully"
