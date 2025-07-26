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

cd "$CIRCUS"

export PKG_CONFIG_LIBDIR="/home/QR2M/compile-circus/STATIC/lib/pkgconfig"
export PKG_CONFIG_PATH="/home/QR2M/compile-circus/STATIC/share/pkgconfig"
export PKG_CONFIG="pkg-config --static"
export CFLAGS="-I/home/QR2M/compile-circus/STATIC/include -O2 -fno-semantic-interposition -Wno-maybe-uninitialized"
export LDFLAGS="-L/home/QR2M/compile-circus/STATIC/lib -lz"
export PATH="/home/QR2M/compile-circus/STATIC/bin:$PATH"

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  needed_files=(
    "zlib.pc"
    "openssl.pc"
    "libnghttp2.pc"
  )

  source "$PROJECT_DIR/check_me_baby.sh" "${needed_files[@]}"
} 2>&1 | tee -a "$LOG_FILE"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat "$LOG_FILE"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  git clone https://github.com/curl/curl.git --depth 1 curl
} 2>&1 | tee -a "$LOG_FILE"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat "$LOG_FILE"
  exit 1
fi

cd curl

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  autoreconf -fi
} 2>&1 | tee -a "$LOG_FILE"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat "$LOG_FILE"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  ./configure \
    --enable-static \
    --disable-shared \
    --prefix="$STATIC_DIR" \
    --with-openssl \
    --with-zlib="$STATIC_DIR" \
    --with-nghttp2="$STATIC_DIR" \
    --enable-ipv6 \
    --without-brotli \
    --without-zstd \
    --without-libidn2 \
    --without-libpsl \
    --without-gnutls \
    --without-mbedtls \
    --without-wolfssl \
    --without-schannel \
    --without-amissl \
    --without-rustls \
    --without-libssh \
    --without-libssh2 \
    --without-gssapi \
    --without-gsasl \
    --disable-tls-srp \
    --disable-ares \
    --disable-threaded-resolver \
    --disable-unix-sockets \
    --without-winidn \
    --disable-docs \
    --disable-manual \
    --disable-verbose \
    --disable-sspi \
    --disable-ldap \
    --disable-ldaps \
    --disable-ipfs \
    --disable-rtsp \
    --without-librtmp \
    --disable-alt-svc \
    --disable-headers-api \
    --disable-hsts \
    --without-ngtcp2 \
    --without-nghttp3 \
    --without-quiche \
    --without-openssl-quic \
    --without-msh3 \
    --with-ca-bundle=/etc/ssl/certs/ca-certificates.crt
} 2>&1 | tee -a "$LOG_FILE"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat "$LOG_FILE"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  make -j"$(nproc)"
} 2>&1 | tee -a "$LOG_FILE"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat "$LOG_FILE"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  make install
} 2>&1 | tee -a "$LOG_FILE"
STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat "$LOG_FILE"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  compiled_files=(
    "libcurl.a"
    "libcurl.pc"
    "curl"
  )

  source "$PROJECT_DIR/check_me_baby.sh" "${compiled_files[@]}"
} 2>&1 | tee -a "$LOG_FILE"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat "$LOG_FILE"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

echo "$(basename "$0") compiled and installed successfully"
