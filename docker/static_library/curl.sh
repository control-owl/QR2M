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
  git clone https://github.com/curl/curl curl
} 2>&1 | tee "$LOG_DIR/curl-01-clone.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat "$LOG_DIR/curl-01-clone.log"
  echo "ERROR - curl - 01/05 - Clone"
  exit 1
fi

cd curl

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  autoreconf -fi
} 2>&1 | tee "$LOG_DIR/curl-02-autoreconf.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/curl-02-autoreconf.log
  echo "ERROR - curl - 02/05 - Clone"
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

#  ./configure \
#  --enable-static \
#  --prefix=$STATIC_DIR \
#    --with-nghttp2 \
#    --with-openssl
##    --with-brotli \
##    --with-idn2 \
##    --with-zstd
#  # --without-libpsl
} 2>&1 | tee "$LOG_DIR/curl-03-configure.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/curl-03-configure.log
  echo "ERROR - curl - 03/05 - Configure"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  make -j"$(nproc)"
} 2>&1 | tee "$LOG_DIR/curl-04-make.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/curl-04-make.log
  echo "ERROR - curl - 04/05 - Compile"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  make install
} 2>&1 | tee "$LOG_DIR/curl-05-install.log"
STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/curl-05-install.log
  echo "ERROR - curl - 05/05 - Install"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

echo "curl compiled and installed successfully"
