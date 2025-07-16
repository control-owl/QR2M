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
export PKG_CONFIG_PATH="$STATIC_DIR/lib/pkgconfig:/usr/lib/pkgconfig:/usr/share/pkgconfig:/usr/lib/x86_64-linux-musl/pkgconfig:/usr/local/lib/pkgconfig"

mkdir -p "$CIRCUS"
mkdir -p "$LOG_DIR"
mkdir -p "$STATIC_DIR"

cd "$CIRCUS"

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  git clone https://github.com/autotools-mirror/gettext.git gettext
} 2>&1 | tee "$LOG_DIR/gettext-01-clone.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat "$LOG_DIR/gettext-01-clone.log"
  echo "ERROR - gettext - 01/05 - Clone"
  exit 1
fi

cd gettext

# {
#   git clone https://github.com/coreutils/gnulib.git gnulib
# } 2>&1 | tee "$LOG_DIR/gnulib-01-clone.log"
# 
# STATUS=${PIPESTATUS[0]}
# if [ "$STATUS" -ne 0 ]; then
#   cat "$LOG_DIR/gnulib-01-clone.log"
#   echo "ERROR - gnulib - 01/05 - Clone"
#   exit 1
# fi
# 
# cd gnulib
# git pull origin master
# cd ..

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

# {
#   ./gnulib-tool --dir=.. --lib=libgettextpo --no-gnu-lib
# } 2>&1 | tee "$LOG_DIR/gnulib-02-autogen.log"
# 
# STATUS=${PIPESTATUS[0]}
# if [ "$STATUS" -ne 0 ]; then
#   cat $LOG_DIR/gnulib-02-autogen.log
#   echo "ERROR - gnulib - 02/05 - Clone"
#   exit 1
# fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  ./autogen.sh
} 2>&1 | tee "$LOG_DIR/gettext-02-autogen.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/gettext-02-autogen.log
  echo "ERROR - gettext - 02/05 - Clone"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  ./configure \
    --enable-static \
    --disable-shared \
    --disable-nls \
    --prefix=$STATIC_DIR
} 2>&1 | tee "$LOG_DIR/gettext-03-configure.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/gettext-03-configure.log
  echo "ERROR - gettext - 03/05 - Configure"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  make -j"$(nproc)"
} 2>&1 | tee "$LOG_DIR/gettext-04-make.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/gettext-04-make.log
  echo "ERROR - gettext - 04/05 - Compile"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  make install
} 2>&1 | tee "$LOG_DIR/gettext-05-install.log"
STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/gettext-05-install.log
  echo "ERROR - gettext - 05/05 - Install"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

echo "gettext compiled and installed successfully"
