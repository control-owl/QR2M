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
export PKG_CONFIG_PATH="/home/QR2M/compile-circus/STATIC/share/pkgconfig:/usr/lib/pkgconfig:/usr/share/pkgconfig:/usr/lib/x86_64-linux-musl/pkgconfig:/usr/local/lib/pkgconfig"
export PKG_CONFIG="pkg-config --static"
export CFLAGS="-I/home/QR2M/compile-circus/STATIC/include -O2 -fno-semantic-interposition -Wno-maybe-uninitialized"
export LDFLAGS="-L/home/QR2M/compile-circus/STATIC/lib -lz -latomic"
export RUSTFLAGS="-C link-arg=-L/home/QR2M/compile-circus/STATIC/lib -C link-arg=-lz -C link-arg=-latomic"

cd "$CIRCUS"

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  pc_files=(
    "freetype2.pc"
    "expat.pc"
    "bzip2.pc"
  )

  source "$PROJECT_DIR/check_me_baby.sh" "${pc_files[@]}"
} 2>&1 | tee "$LOG_DIR/appstream-verify.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat "$LOG_DIR/appstream-verify.log"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  git clone https://github.com/fontconfig/fontconfig.git --depth 1 fontconfig
} 2>&1 | tee "$LOG_DIR/fontconfig-01-clone.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat "$LOG_DIR/fontconfig-01-clone.log"
  exit 1
fi

cd fontconfig

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  meson setup builddir \
    -Dprefix="$STATIC_DIR" \
    -Ddefault_library=static \
    -Ddoc=disabled \
    -Ddoc-txt=disabled \
    -Ddoc-man=disabled \
    -Ddoc-pdf=disabled \
    -Ddoc-html=disabled \
    -Dnls=disabled \
    -Dtests=disabled \
    -Dtools=disabled \
    -Dcache-build=disabled \
    -Diconv=disabled \
    -Dxml-backend=expat \
    -Dfontations=disabled \
    -Ddefault-hinting=slight \
    -Ddefault-sub-pixel-rendering=none \
    -Dbitmap-conf=no-except-emoji \
    -Ddefault-fonts-dirs=yes \
    -Dadditional-fonts-dirs=yes \
    -Dcache-dir=default \
    -Dtemplate-dir=default \
    -Dbaseconfig-dir=default \
    -Dconfig-dir=default \
    -Dxml-dir=default \
    -Dbuildtype=release
} 2>&1 | tee "$LOG_DIR/fontconfig-02-setup.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat "$LOG_DIR/fontconfig-02-setup.log"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  ninja -C builddir
} 2>&1 | tee "$LOG_DIR/fontconfig-03-compile.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/fontconfig-03-compile.log
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  ninja -C builddir install
} 2>&1 | tee "$LOG_DIR/fontconfig-04-install.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/fontconfig-04-install.log
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

echo "fontconfig compiled and installed successfully"