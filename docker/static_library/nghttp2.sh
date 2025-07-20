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

export PKG_CONFIG_LIBDIR="/home/QR2M/compile-circus/STATIC/lib/pkgconfig"
export PKG_CONFIG_PATH="/home/QR2M/compile-circus/STATIC/share/pkgconfig"
export PKG_CONFIG="pkg-config --static"
export CFLAGS="-I/home/QR2M/compile-circus/STATIC/include -O2 -fno-semantic-interposition -Wno-maybe-uninitialized"
export LDFLAGS="-L/home/QR2M/compile-circus/STATIC/lib -lz -latomic"
export RUSTFLAGS="-C link-arg=-L/home/QR2M/compile-circus/STATIC/lib -C link-arg=-lz -C link-arg=-latomic"

cd "$CIRCUS"

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  git clone https://github.com/nghttp2/nghttp2.git --depth 1 nghttp2
} 2>&1 | tee "$LOG_DIR/nghttp2-01-clone.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat "$LOG_DIR/nghttp2-01-clone.log"
  echo "ERROR - nghttp2 - 01/07 - Clone"
  exit 1
fi

cd nghttp2

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  autoreconf -i
} 2>&1 | tee "$LOG_DIR/nghttp2-02-autoreconf.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/nghttp2-02-autoreconf.log
  echo "ERROR - nghttp2 - 02/07 - autoreconf"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  automake
} 2>&1 | tee "$LOG_DIR/nghttp2-03-automake.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/nghttp2-03-automake.log
  echo "ERROR - nghttp2 - 03/07 - automake"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  autoconf
} 2>&1 | tee "$LOG_DIR/nghttp2-04-autoconf.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/nghttp2-04-autoconf.log
  echo "ERROR - nghttp2 - 04/07 - autoconf"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  ./configure \
    --enable-static \
    --disable-shared \
    --prefix="$STATIC_DIR" \
    --enable-lib-only \
    --disable-werror \
    --disable-debug \
    --disable-threads \
    --disable-http3 \
    --without-libxml2 \
    --without-jansson \
    --with-zlib="$STATIC_DIR" \
    --without-libevent-openssl \
    --without-libcares \
    --without-wolfssl \
    --with-openssl="$STATIC_DIR" \
    --without-libev \
    --without-jemalloc \
    --without-systemd \
    --without-mruby \
    --without-neverbleed \
    --without-libngtcp2 \
    --without-libnghttp3 \
    --without-libbpf \
    --without-libbrotlienc \
    --without-libbrotlidec
} 2>&1 | tee "$LOG_DIR/nghttp2-05-configure.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/nghttp2-05-configure.log
  echo "ERROR - nghttp2 - 05/07 - Configure"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  make -j"$(nproc)"
} 2>&1 | tee "$LOG_DIR/nghttp2-06-make.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/nghttp2-06-make.log
  echo "ERROR - nghttp2 - 06/07 - make"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  make install
} 2>&1 | tee "$LOG_DIR/nghttp2-07-install.log"
STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/nghttp2-07-install.log
  echo "ERROR - nghttp2 - 07/07 - Install"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

echo "nghttp2 compiled and installed successfully"
