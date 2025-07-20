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
  git clone https://github.com/ximion/appstream.git --depth 1 appstream
} 2>&1 | tee "$LOG_DIR/appstream-01-clone.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat "$LOG_DIR/appstream-01-clone.log"
  echo "ERROR - appstream - 01/04 - Clone"
  exit 1
fi

cd appstream

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  export PKG_CONFIG_PATH="$STATIC_DIR/lib/pkgconfig"
  export PKG_CONFIG="pkg-config --static"
  export RUSTFLAGS="-C link-arg=-L$STATIC_DIR/lib -C link-arg=-lz -C link-arg=-latomic"
  export CFLAGS="-I$STATIC_DIR/include -O2 -fno-semantic-interposition -Wno-maybe-uninitialized"
  export LDFLAGS="-L$STATIC_DIR/lib -lz -latomic"
  PKG_CONFIG_LIBDIR="$STATIC_DIR/lib/pkgconfig" meson setup builddir \
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
    -Dstatic-analysis=false 
} 2>&1 | tee "$LOG_DIR/appstream-02-setup.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat "$LOG_DIR/appstream-02-setup.log"
  echo "ERROR - appstream - 02/04 - Setup"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  ninja -C builddir
} 2>&1 | tee "$LOG_DIR/appstream-03-compile.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/appstream-03-compile.log
  echo "ERROR - appstream - 03/04 - Compile"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  ninja -C builddir install
} 2>&1 | tee "$LOG_DIR/appstream-04-install.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/appstream-04-install.log
  echo "ERROR - appstream - 04/04 - Install"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

echo "appstream compiled and installed successfully"