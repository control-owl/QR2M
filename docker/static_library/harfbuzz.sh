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
  git clone https://github.com/harfbuzz/harfbuzz.git harfbuzz
} 2>&1 | tee "$LOG_DIR/harfbuzz-01-clone.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat "$LOG_DIR/harfbuzz-01-clone.log"
  echo "ERROR - harfbuzz - 01/04 - Clone"
  exit 1
fi

cd harfbuzz

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  meson setup builddir \
    -Dprefix="$STATIC_DIR" \
    -Ddefault_library=static \
    -Dfreetype=enabled \
    -Dcairo=enabled \
    -Dglib=enabled \
    -Dgobject=disabled \
    -Dchafa=disabled \
    -Dicu=disabled \
    -Dgraphite2=disabled \
    -Dfontations=disabled \
    -Dgdi=disabled \
    -Ddirectwrite=disabled \
    -Dcoretext=disabled \
    -Dharfrust=disabled \
    -Dwasm=disabled \
    -Dtests=disabled \
    -Dintrospection=disabled \
    -Ddocs=disabled \
    -Ddoc_tests=false \
    -Dutilities=disabled \
    -Dbenchmark=disabled \
    -Dicu_builtin=false \
    -Dwith_libstdcxx=false \
    -Dexperimental_api=false \
    -Dragel_subproject=false \
    -Dbuildtype=release \
    -Db_ndebug=true
} 2>&1 | tee "$LOG_DIR/harfbuzz-02-setup.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/harfbuzz-02-setup.log
  echo "ERROR - harfbuzz - 02/04 - Setup"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  ninja -C builddir
} 2>&1 | tee "$LOG_DIR/harfbuzz-03-compile.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/harfbuzz-03-compile.log
  echo "ERROR - harfbuzz - 03/04 - Compile"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  ninja -C builddir install
} 2>&1 | tee "$LOG_DIR/harfbuzz-04-install.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/harfbuzz-04-install.log
  echo "ERROR - harfbuzz - 04/04 - Install"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

echo "harfbuzz compiled and installed successfully"