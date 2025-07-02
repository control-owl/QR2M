#!/bin/sh
set -e


echo "Environment variables:"
[ -z "$APP_NAME" ] && { echo "Error: Environment variable APP_NAME is not set"; exit 1; }
[ -z "$BUILD_PATH" ] && { echo "Error: Environment variable BUILD_PATH is not set"; exit 1; }
[ -z "$OUTPUT_DIR" ] && { echo "Error: Environment variable OUTPUT_DIR is not set"; exit 1; }
[ -z "$FEATURES" ] && { echo "Error: Environment variable FEATURES is not set"; exit 1; }
[ -z "$TARGET" ] && { echo "Error: Environment variable TARGET is not set"; exit 1; }
echo "APP_NAME=$APP_NAME"
echo "BUILD_PATH=$BUILD_PATH"
echo "OUTPUT_DIR=$OUTPUT_DIR"
echo "FEATURES=$FEATURES"
echo "TARGET=$TARGET"
echo "OUTPUT=$OUTPUT"
echo "Running docker.sh from $(pwd)"


echo "Setting up Alpine repositories..."
echo "https://dl-cdn.alpinelinux.org/alpine/v3.22/main" > /etc/apk/repositories
echo "https://dl-cdn.alpinelinux.org/alpine/v3.22/community" >> /etc/apk/repositories


echo "Installing fucking dependencies"
apk update
apk add --no-cache \
  bash curl build-base git cmake ninja meson \
    pkgconf-dev musl-dev \
    gettext-dev gettext-static \
    openssl-dev openssl-libs-static \
    glib-dev glib-static \
    cairo-dev cairo-static \
    harfbuzz-dev harfbuzz-static \
    libxml2-dev libxml2-static \
    libx11-dev libxrandr-dev libxrender-dev libxext-dev \
    libxfixes-dev libxcursor-dev libxi-dev \
    pango-dev \
    fontconfig-dev \
    gdk-pixbuf-dev \
    librsvg-dev \
    vulkan-loader-dev vulkan-tools \
    zlib-static \
    xz-dev \
    glslang glslang-dev glslang-static \
    shaderc-static shaderc-dev \
    flex-dev flex-libs \
    libxkbcommon-dev libxkbcommon-static \
    libxdamage-dev \
    bison


echo "START COMPILE CIRCUS"
mkdir -p /compile-circus

compile_gtk4() {
  cd /compile-circus
  git clone https://gitlab.gnome.org/GNOME/gtk.git --depth 1
  cd gtk
  mkdir builddir
  echo "START MESON SETUP GTK4"
  if ! meson setup builddir \
    -Dmedia-gstreamer=disabled \
    -Dprint-cpdb=disabled \
    -Dprint-cups=disabled \
    -Dvulkan=disabled \
    -Dcloudproviders=disabled \
    -Dsysprof=disabled \
    -Dtracker=disabled \
    -Dcolord=disabled \
    -Daccesskit=disabled \
    -Dintrospection=disabled \
    -Ddocumentation=false \
    -Dscreenshots=false \
    -Dman-pages=false \
    -Dbuild-demos=false \
    -Dbuild-testsuite=false \
    -Dbuild-examples=false \
    -Dbuild-tests=false; then
      echo "START MESON SETUP GTK4 FAIL"
      echo "LOG /compile-circus/gtk/builddir/meson-logs/meson-log.txt:"
      cat /compile-circus/gtk/builddir/meson-logs/meson-log.txt
      echo "END MESON SETUP GTK4 FAIL"

      echo "START MESON SETUP GTK4 subprojects FAIL"
      echo "LOG /compile-circus/gtk/subprojects/sysprof/meson.build:"
      cat /compile-circus/gtk/subprojects/sysprof/meson.build
      echo "END MESON SETUP GTK4 subprojects FAIL"
      exit 1
  else
    echo "END MESON SETUP GTK4: Success"
  fi

  echo "START MESON INSTALL GTK4"
  if meson install -C "builddir"; then
    echo "END MESON INSTALL GTK4: Success"
  else
      echo "START MESON INSTALL GTK4 FAIL"
      echo "LOG /compile-circus/gtk/builddir/meson-logs/meson-log.txt:"
      cat "/compile-circus/gtk/builddir/meson-logs/meson-log.txt"
      exit 1
  fi
}

compile_svg() {
  cd /compile-circus
  git clone https://gitlab.gnome.org/GNOME/librsvg.git
  cd librsvg
  if ! meson setup builddir \
    -Dtests=false \
    -Ddocs=disabled \
    -Dvala=disabled \
    -Davif=disabled; then
      echo "START MESON SETUP SVG FAIL"
      echo "LOG /compile-circus/librsvg/builddir/meson-logs/meson-log.txt:"
      cat /compile-circus/librsvg/builddir/meson-logs/meson-log.txt
      echo "END MESON SETUP SVG FAIL"
  else
    echo "END MESON SETUP SVG: Success"
  fi

  if ! meson compile -C builddir; then
    echo "START MESON COMPILE SVG FAIL"
    echo "LOG /compile-circus/librsvg/builddir/meson-logs/meson-log.txt:"
    cat /compile-circus/librsvg/builddir/meson-logs/meson-log.txt
    echo "END MESON COMPILE SVG FAIL"
  else
    echo "END MESON COMPILE SVG: Success"
  fi

  if ! meson install -C builddir; then
    echo "START MESON INSTALL SVG FAIL"
    echo "LOG /compile-circus/librsvg/builddir/meson-logs/meson-log.txt:"
    cat /compile-circus/librsvg/builddir/meson-logs/meson-log.txt
    echo "END MESON INSTALL SVG FAIL"
  else
    echo "END MESON INSTALL SVG: Success"
  fi
}

compile_adwaita() {
  cd /compile-circus

  echo "Installing cargo-c for Rust C ABI compatibility..."
  cargo install cargo-c || { echo "Error: Failed to install cargo-c"; exit 1; }
  echo "Verifying cargo-c installation..."
  cargo-cbuild --version || { echo "Error: cargo-cbuild not found or not executable"; exit 1; }

  git clone https://gitlab.gnome.org/GNOME/libadwaita.git
  cd libadwaita
  if ! meson setup builddir \
    -Dexamples=false \
    -Dtests=false; then
    echo "START MESON SETUP ADWAITA FAIL"
    echo "LOG /compile-circus/libadwaita/builddir/meson-logs/meson-log.txt:"
    cat /compile-circus/libadwaita/builddir/meson-logs/meson-log.txt
    echo "END MESON SETUP ADWAITA FAIL"
  else
    echo "END MESON SETUP ADWAITA: Success"
  fi

  if ! ninja -C builddir; then
    echo "START NINJA COMPILE ADWAITA FAIL"
    echo "LOG /compile-circus/libadwaita/builddir/meson-logs/meson-log.txt:"
    cat /compile-circus/libadwaita/builddir/meson-logs/meson-log.txt
    echo "END NINJA COMPILE ADWAITA FAIL"
  else
    echo "END NINJA COMPILE ADWAITA: Success"
  fi

  if ! ninja -C builddir install; then
    echo "START NINJA INSTALL ADWAITA FAIL"
    echo "LOG /compile-circus/libadwaita/builddir/meson-logs/meson-log.txt:"
    cat /compile-circus/libadwaita/builddir/meson-logs/meson-log.txt
    echo "END NINJA INSTALL ADWAITA FAIL"
  else
    echo "END NINJA INSTALL ADWAITA: Success"
  fi
}

echo "START Compiling librsvg..."
compile_svg || { echo "FAIL: librsvg compilation failed"; exit 1; }

echo "START Compiling libadwaita..."
compile_adwaita || { echo "FAIL: libadwaita compilation failed"; exit 1; }

echo "START Compiling GTK4..."
compile_gtk4 || { echo "FAIL: GTK4 compilation failed"; exit 1; }

echo "Verifying installed packages..."
apk list -I | grep -E "gtk4.0-dev|libadwaita-dev|pkgconf|file" || {
  echo "Error: Key packages not installed"
  exit 1
}


echo "Checking for static libraries..."
ls -l /usr/lib/libssl.a /usr/lib/libcrypto.a || {
  echo "Warning: Static libraries libssl.a or libcrypto.a not found, checking alternatives"
  ls -l /usr/lib/*ssl*.a /usr/lib/*crypto*.a || echo "No static libraries found"
}


echo "Capturing .pc file paths..."
GTK4=$(apk info -L gtk4.0-dev | grep -E '/gtk4\.pc$' | sed "s|^|/|")
ADWAITA=$(apk info -L libadwaita-dev | grep -E '/libadwaita-1\.pc$' | sed "s|^|/|")
echo "GTK4=$GTK4"
echo "ADWAITA=$ADWAITA"
[ -n "$GTK4" ] || { echo "Error: gtk4.pc not found in gtk4.0-dev"; exit 1; }
[ -n "$ADWAITA" ] || { echo "Error: libadwaita-1.pc not found in libadwaita-dev"; exit 1; }


echo "Renaming .pc files..."
cp "$GTK4" "$(dirname "$GTK4")/gtk-4.0.pc" || { echo "Error: Failed to rename gtk4.pc"; exit 1; }
cp "$ADWAITA" "$(dirname "$ADWAITA")/libadwaita-1.0.pc" || { echo "Error: Failed to rename libadwaita-1.pc"; exit 1; }


echo "Set PKG_CONFIG_PATH"
export PKG_CONFIG_PATH="/usr/lib/pkgconfig:/usr/share/pkgconfig:/usr/lib/x86_64-linux-musl/pkgconfig:/usr/local/lib/pkgconfig"
GTK_PC_PATH=$(dirname "$GTK4")
LIBADWAITA_PC_PATH=$(dirname "$ADWAITA")
export PKG_CONFIG_PATH="$GTK_PC_PATH:$LIBADWAITA_PC_PATH:$PKG_CONFIG_PATH"
echo "PKG_CONFIG_PATH=$PKG_CONFIG_PATH"


echo "Checking pkg-config versions..."
pkg-config --modversion gtk-4.0 || { echo "Error: gtk-4.0 not found"; exit 1; }
pkg-config --modversion libadwaita-1.0 || { echo "Error: libadwaita-1.0 not found"; exit 1; }
pkg-config --libs --cflags openssl || { echo "Error: OpenSSL pkg-config not found"; exit 1; }


echo "Install Rust MUSL target"
rustup target add x86_64-unknown-linux-musl


echo "Set environment variables for build"
export PKG_CONFIG_ALLOW_CROSS=1
#export CFLAGS="-I/usr/include"
#export LDFLAGS="-L/usr/lib -L/usr/lib/x86_64-linux-musl"
# export CFLAGS="-static -O2 -fPIC"
export CFLAGS="-static"
export LDFLAGS="-static"
export OPENSSL_DIR=/usr
export OPENSSL_LIB_DIR=/usr/lib
export OPENSSL_INCLUDE_DIR=/usr/include
export OPENSSL_STATIC=1
export RUSTFLAGS="-C target-feature=+crt-static -C link-arg=-L/usr/lib -C link-arg=-lssl -C link-arg=-lcrypto -C link-arg=-static"
# export RUSTFLAGS="-C target-feature=+crt-static -C linker=musl-gcc"


echo "Building project..."
cd /compile-circus
git clone https://github.com/control-owl/QR2M
cd QR2M
cargo build --release --target "$TARGET" --features "$FEATURES" --locked -vv && echo "Cargo build done" || exit 1
# cargo test --release --locked - --no-fail-fast --target "$TARGET" --features "$FEATURES" && echo "Cargo test done"


echo "Listing build directory:"
ls -l "$BUILD_PATH"


echo "Checking binary:"
export BIN="$BUILD_PATH/$APP_NAME"
[ -f "$BIN" ] || { echo "Error: Binary not found at $BIN"; exit 1; }
file "$BIN"
ldd "$BIN"
chmod +x "$BIN"


if [ "$OUTPUT" = "true" ]; then
  echo "Copying files to $OUTPUT_DIR..."
  mkdir -p "$OUTPUT_DIR"
  cp "$BIN" "$OUTPUT_DIR" || { echo "Error: Failed to copy $BIN to $OUTPUT_DIR"; exit 1; }
  chown 1001:1001 "$OUTPUT_DIR/$APP_NAME" || { echo "Error: Failed to change ownership of $OUTPUT_DIR/$APP_NAME"; exit 1; }

  echo "Listing output directory:"
  ls -l "$OUTPUT_DIR"
fi


# ALL GTK4 deps in Arch Linux
# Dependencies (61)
# adwaita-fonts
# adwaita-icon-theme
# at-spi2-core
# bash
# cairo
# dconf
# desktop-file-utils
# fontconfig
# fribidi
# gcc-libs
# gdk-pixbuf2
# glib2
# glibc
# graphene
# gst-plugins-bad-libs
# gst-plugins-base-libs
# gstreamer
# gtk-update-icon-cache
# harfbuzz
# iso-codes
# libcloudproviders
# libcolord
# libcups
# libegl (libglvnd)
# libepoxy
# libgl (libglvnd)
# libjpeg-turbo
# libpng
# librsvg
# libtiff
# libx11
# libxcursor
# libxdamage
# libxext
# libxfixes
# libxi
# libxinerama
# libxkbcommon
# libxrandr
# libxrender
# pango
# shared-mime-info
# tinysparql
# vulkan-icd-loader
# wayland
# evince (optional) - Default print preview command
# cantarell-fonts (make)
# docbook-xsl (make)
# gi-docgen (make)
# git (make)
# glib2-devel (make)
# gobject-introspection (make)
# hicolor-icon-theme (make)
# libsysprof-capture (make)
# meson (make)
# python-docutils (make)
# python-gobject (make)
# sassc (make)
# shaderc (make)
# vulkan-headers (make)
# wayland-protocols (make)