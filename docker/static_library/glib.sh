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
  git clone --depth 1 --no-tags https://gitlab.gnome.org/GNOME/glib.git glib
  git -C glib submodule update --init
  meson subprojects download --sourcedir glib
  # rm glib/subprojects/*.wrap
  # mv glib/subprojects/ .
} 2>&1 | tee "$LOG_DIR/glib-01-clone.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat "$LOG_DIR/glib-01-clone.log"
  echo "ERROR - glib - 01/04 - Clone"
  exit 1
fi

cd glib

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  meson setup builddir \
    -Dprefix="$STATIC_DIR" \
    -Ddefault_library=static \
    -Dselinux=disabled \
    -Dxattr=false \
    -Dlibmount=disabled \
    -Dman-pages=false \
    -Ddtrace=disabled \
    -Dsystemtap=disabled \
    -Dsysprof=disabled \
    -Ddocumentation=false \
    -Dbsymbolic_functions=true \
    -Dforce_posix_threads=false \
    -Dtests=false \
    -Dinstalled_tests=false \
    -Dnls=enabled \
    -Doss_fuzz=disabled \
    -Dglib_debug=disabled \
    -Dglib_assert=false \
    -Dglib_checks=false \
    -Dlibelf=disabled \
    -Dmultiarch=false \
    -Dintrospection=disabled \
    -Dfile_monitor_backend=inotify \
    --buildtype=release
} 2>&1 | tee "$LOG_DIR/glib-02-setup.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/glib-02-setup.log
  echo "ERROR - glib - 02/04 - Setup"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  ninja -C builddir
} 2>&1 | tee "$LOG_DIR/glib-03-compile.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/glib-03-compile.log
  echo "ERROR - glib - 03/04 - Compile"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  ninja -C builddir install
} 2>&1 | tee "$LOG_DIR/glib-04-install.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/glib-04-install.log
  echo "ERROR - glib - 04/04 - Install"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

echo "glib compiled and installed successfully"