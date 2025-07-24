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
LOG_FILE="$LOG_DIR/$(basename "$0").log"
STATIC_DIR="$CIRCUS/STATIC"

mkdir -p "$CIRCUS"
mkdir -p "$LOG_DIR"
mkdir -p "$STATIC_DIR"

export PKG_CONFIG_LIBDIR="/home/QR2M/compile-circus/STATIC/lib/pkgconfig"
export PKG_CONFIG_PATH="/home/QR2M/compile-circus/STATIC/share/pkgconfig:/usr/lib/pkgconfig:/usr/share/pkgconfig:/usr/lib/x86_64-linux-musl/pkgconfig:/usr/local/lib/pkgconfig"
export PKG_CONFIG="pkg-config --static"
export CFLAGS="-I/home/QR2M/compile-circus/STATIC/include -O2 -fno-semantic-interposition -Wno-maybe-uninitialized"
export LDFLAGS="-L/home/QR2M/compile-circus/STATIC/lib -lz -latomic"
export RUSTFLAGS="-C link-arg=-L/home/QR2M/compile-circus/STATIC/lib -C link-arg=-lz -C link-arg=-latomic"

cd "$CIRCUS"

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  pc_files=()

  source "$PROJECT_DIR/check_me_baby.sh" "${pc_files[@]}"
} 2>&1 | tee "$LOG_DIR/appstream-verify.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat "$LOG_DIR/appstream-verify.log"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  git clone https://github.com/libjpeg-turbo/libjpeg-turbo.git --depth 1 libjpeg-turbo
} 2>&1 | tee "$LOG_DIR/libjpeg-turbo-01-clone.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat "$LOG_DIR/libjpeg-turbo-01-clone.log"
  exit 1
fi

cd libjpeg-turbo

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.


{
  mkdir -p builddir
  cd builddir
  cmake -G"Unix Makefiles" -DCMAKE_INSTALL_PREFIX=$STATIC_DIR -DENABLE_SHARED=0 ..
} 2>&1 | tee "$LOG_DIR/libjpeg-turbo-03-configure.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/libjpeg-turbo-03-configure.log
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  make -j"$(nproc)" -C builddir
} 2>&1 | tee "$LOG_DIR/libjpeg-turbo-04-make.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/libjpeg-turbo-04-make.log
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  make install -C builddir
} 2>&1 | tee "$LOG_DIR/libjpeg-turbo-05-install.log"
STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/libjpeg-turbo-05-install.log
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

echo "libjpeg-turbo compiled and installed successfully"
