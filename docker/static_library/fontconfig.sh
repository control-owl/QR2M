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
  git clone https://github.com/fontconfig/fontconfig.git fontconfig
} 2>&1 | tee "$LOG_DIR/fontconfig-01-clone.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat "$LOG_DIR/fontconfig-01-clone.log"
  echo "ERROR - fontconfig - 01/05 - Clone"
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
  echo "ERROR - fontconfig - 02/04 - Setup"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  ninja -C builddir
} 2>&1 | tee "$LOG_DIR/fontconfig-03-compile.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/fontconfig-03-compile.log
  echo "ERROR - fontconfig - 03/04 - Compile"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  ninja -C builddir install
} 2>&1 | tee "$LOG_DIR/fontconfig-04-install.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/fontconfig-04-install.log
  echo "ERROR - fontconfig - 04/04 - Install"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

echo "fontconfig compiled and installed successfully"