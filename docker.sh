#!/bin/sh
set -e


echo "Environment variables:"
[ -z "$APP_NAME" ] && { echo "Error: Environment variable APP_NAME is not set"; exit 1; }
[ -z "$APP_PATH" ] && { echo "Error: Environment variable APP_PATH is not set"; exit 1; }
[ -z "$OUTPUT_DIR" ] && { echo "Error: Environment variable OUTPUT_DIR is not set"; exit 1; }
[ -z "$FEATURES" ] && { echo "Error: Environment variable FEATURES is not set"; exit 1; }
[ -z "$TARGET" ] && { echo "Error: Environment variable TARGET is not set"; exit 1; }
echo "APP_NAME=$APP_NAME"
echo "APP_PATH=$APP_PATH"
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
    pango-dev \
    harfbuzz-dev harfbuzz-static \
    fontconfig-dev \
    gdk-pixbuf-dev \
    libxml2-dev libxml2-static \
    libx11-dev libxrandr-dev libxrender-dev libxext-dev \
    libxfixes-dev libxcursor-dev libxi-dev \
    libadwaita-dev \
    librsvg-dev \
    vulkan-loader-dev \
    zlib-static \
    xz-dev \
    glslang glslang-dev glslang-static


echo "START COMPILE CIRCUS"
mkdir -p /compile-circus && cd /compile-circus


# Install glslc (shader compiler)
git clone https://github.com/KhronosGroup/glslang.git --depth 1
cd glslang
cmake -S . -B build -DCMAKE_BUILD_TYPE=Release -DENABLE_OPT=OFF -DBUILD_GLSLC=ON
cmake --build build -j"$(nproc)"
ls -l build/StandAlone/
cp build/StandAlone/glslc /usr/local/bin/
glslc --version || { echo "ERROR: glslc build failed"; exit 1; }

cd /compile-circus
git clone https://gitlab.gnome.org/GNOME/gtk.git --depth 1
cd gtk
mkdir builddir
meson setup builddir
meson install -C builddir


echo "END COMPILE CIRCUS"


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
export CFLAGS="-static -O2 -fPIC"
export LDFLAGS="-static"
export OPENSSL_DIR=/usr
export OPENSSL_LIB_DIR=/usr/lib
export OPENSSL_INCLUDE_DIR=/usr/include
export OPENSSL_STATIC=1
#export RUSTFLAGS="-C target-feature=+crt-static -C link-arg=-L/usr/lib -C link-arg=-lssl -C link-arg=-lcrypto -C link-arg=-static"
export RUSTFLAGS="-C target-feature=+crt-static -C linker=musl-gcc"

echo "Building project..."
cargo build --release --target "$TARGET" --features "$FEATURES" --locked -vv
#cargo test --release --locked --verbose --no-fail-fast --target "$TARGET" --features "$FEATURES"


echo "Listing build directory:"
ls -l "$APP_PATH"


echo "Checking binary:"
export BIN="$APP_PATH/$APP_NAME"
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
