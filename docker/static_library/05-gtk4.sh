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

#export PKG_CONFIG_LIBDIR="/home/QR2M/compile-circus/STATIC/lib/pkgconfig"
export PKG_CONFIG_PATH="/home/QR2M/compile-circus/STATIC/lib/pkgconfig:/home/QR2M/compile-circus/STATIC/lib64/pkgconfig:/home/QR2M/compile-circus/STATIC/share/pkgconfig"
export CFLAGS="-I/home/QR2M/compile-circus/STATIC/include -O2 -fno-semantic-interposition -Wno-maybe-uninitialized -fPIC"
export CXXFLAGS="-I/home/QR2M/compile-circus/STATIC/include -O2 -fno-semantic-interposition -Wno-deprecated-declarations -fPIC"
export LDFLAGS="-L/home/QR2M/compile-circus/STATIC/lib -L/home/QR2M/compile-circus/STATIC/lib64 -lXrandr -lXrender -lX11 -lXext -lxkbcommon -lXi -lffi -lssl -lcrypto -lcurl"
export PATH="/home/QR2M/compile-circus/STATIC/bin:$PATH"
export PKG_CONFIG="pkg-config --static"

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  needed_files=(
    "glib-2.0.pc"
    "cairo.pc"
    "pango.pc"
    "fribidi.pc"
    "epoxy.pc"
    "graphene-1.0.pc"
    "gdk-pixbuf-2.0.pc"
    "fontconfig.pc"
    "freetype2.pc"
    "libxml-2.0.pc"
    "libtiff-4.pc"
    "libpng16.pc"
    "pixman-1.pc"
    "libssl.pc"
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
  git clone https://gitlab.gnome.org/GNOME/gtk.git --depth 1 --verbose gtk4
} 2>&1 | tee -a "$LOG_FILE"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat "$LOG_FILE"
  exit 1
fi

cd gtk4

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  meson setup builddir \
    --prefix=$STATIC_DIR \
    -Ddefault_library=static \
    -Dx11-backend=true \
    -Dwayland-backend=false \
    -Dbroadway-backend=false \
    -Dwin32-backend=false \
    -Dmacos-backend=false \
    -Dandroid-backend=false \
    \
    -Dmedia-gstreamer=disabled \
    -Dprint-cpdb=disabled \
    -Dprint-cups=disabled \
    \
    -Dvulkan=disabled \
    -Dcloudproviders=disabled \
    -Dsysprof=disabled \
    -Dtracker=disabled \
    -Dcolord=disabled \
    -Df16c=enabled \
    -Daccesskit=disabled \
    -Dandroid-runtime=disabled \
    \
    -Dintrospection=disabled \
    \
    -Ddocumentation=false \
    -Dscreenshots=false \
    -Dman-pages=false \
    \
    -Dprofile=auto \
    -Dbuild-demos=false \
    -Dbuild-testsuite=false \
    -Dbuild-examples=false \
    -Dbuild-tests=false
}  2>&1 | tee -a "$LOG_FILE"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat "$LOG_FILE"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  ninja -C builddir
} 2>&1 | tee -a "$LOG_FILE"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat "$LOG_FILE"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  ninja -C builddir install
} 2>&1 | tee -a "$LOG_FILE"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat "$LOG_FILE"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

if [ ! -d "$STATIC_DIR/lib/pkgconfig" ]; then
  mkdir -p "$STATIC_DIR/lib/pkgconfig"
fi

if [ -f "$STATIC_DIR/lib/pkgconfig/gtk4.pc" ]; then
  cp "$STATIC_DIR/lib/pkgconfig/gtk4.pc" "$STATIC_DIR/lib/pkgconfig/gtk-4.pc"
  sed -i "s|Libs:.*|Libs: -L$STATIC_DIR/lib -lgtk-4|" "$STATIC_DIR/lib/pkgconfig/gtk-4.pc"
  sed -i "s|Cflags:.*|Cflags: -I$STATIC_DIR/include/gtk-4.0 -I$STATIC_DIR/include|" "$STATIC_DIR/lib/pkgconfig/gtk-4.pc"
else
  echo "Error: gtk4.pc not found in $STATIC_DIR/lib/pkgconfig"
  if [ -f "$STATIC_DIR/lib/pkgconfig/gtk-4.pc" ]; then
    echo "gtk-4.pc found"
  else
    echo "No gtk*4.pc found"
    exit 1
  fi
fi

if [ -f "$STATIC_DIR/lib/pkgconfig/gtk4.pc" ]; then
  if pkg-config --modversion "$STATIC_DIR/lib/pkgconfig/gtk4.pc" > /dev/null 2>&1; then
    echo "Verified: gtk4.pc is valid"
  else
    echo "Error: pkg-config cannot validate gtk4.pc"
    if [ -f "$STATIC_DIR/lib/pkgconfig/gtk4.pc" ]; then
      cat "$STATIC_DIR/lib/pkgconfig/gtk4.pc"
    fi
    exit 1
  fi
fi

if [ -f "$STATIC_DIR/lib/libgtk-4.so" ]; then
  ar rcs "$STATIC_DIR/lib/libgtk-4.a" "$STATIC_DIR/lib/libgtk-4.so"
  if [ "$?" -ne 0 ]; then
    echo "Error: Failed to create libgtk-4.a from libgtk-4.so"
    exit 1
  fi
else
  echo "Error: Shared library $STATIC_DIR/lib/libgtk-4.so not found"
  if [ -d "$STATIC_DIR/lib" ]; then
    ls -l "$STATIC_DIR/lib/libgtk*"
  fi
  exit 1
fi

if [ ! -f "$STATIC_DIR/lib/libgtk-4.a" ]; then
  echo "Error: Library $STATIC_DIR/lib/libgtk-4.a not found after creation"
  if [ -d "$STATIC_DIR/lib" ]; then
    ls -l "$STATIC_DIR/lib/libgtk*.a"
  fi
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.


{
  compiled_files=(
    "libgtk-4.a"
    "gtk4.pc"
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
