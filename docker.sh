#!/bin/sh
set -e


echo "Checking environment variables..."
[ -z "$APP_NAME" ] && { echo "Error: Environment variable APP_NAME is not set"; exit 1; }
[ -z "$APP_PATH" ] && { echo "Error: Environment variable APP_PATH is not set"; exit 1; }
[ -z "$OUTPUT_DIR" ] && { echo "Error: Environment variable OUTPUT_DIR is not set"; exit 1; }
[ -z "$FEATURES" ] && { echo "Error: Environment variable FEATURES is not set"; exit 1; }
[ -z "$TARGET" ] && { echo "Error: Environment variable TARGET is not set"; exit 1; }


echo "Environment variables:"
echo "APP_NAME=$APP_NAME"
echo "APP_PATH=$APP_PATH"
echo "OUTPUT_DIR=$OUTPUT_DIR"
echo "FEATURES=$FEATURES"
echo "TARGET=$TARGET"
echo "Running docker.sh from $(pwd)"


echo "Setting up Alpine repositories..."
echo "https://dl-cdn.alpinelinux.org/alpine/v3.22/main" > /etc/apk/repositories
echo "https://dl-cdn.alpinelinux.org/alpine/v3.22/community" >> /etc/apk/repositories


echo "Installing system dependencies"
apk update
apk add --no-cache \
  musl-dev \
  pkgconf \
  pkgconf-dev \
  git \
  cmake \
  meson \
  ninja \
  glib-dev \
  gtk4.0-dev \
  libadwaita-dev \
  cairo-dev \
  pango-dev \
  gdk-pixbuf-dev \
  harfbuzz-dev \
  graphene-dev \
  vulkan-loader-dev \
  fontconfig-dev \
  freetype-dev \
  openssl-dev \
  curl \
  file

echo "Verifying installed packages..."
apk list -I | grep -E "gtk4.0-dev|libadwaita-dev|pkgconf|file" || {
  echo "Error: Key packages not installed"
  exit 1
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

rustup target add x86_64-unknown-linux-musl

echo "Set environment variables for build"
export PKG_CONFIG_ALLOW_CROSS=1
export CFLAGS="-I/usr/include"
export LDFLAGS="-L/usr/lib"
export OPENSSL_DIR=/usr
export OPENSSL_LIB_DIR=/usr/lib
export OPENSSL_INCLUDE_DIR=/usr/include
export RUSTFLAGS="-C target-feature=-crt-static -C link-args=-Wl,-rpath,/usr/lib"

echo "Building project..."
cargo build --release --target "$TARGET" --features "$FEATURES" --locked --verbose
cargo test --release --locked --verbose --no-fail-fast --target "$TARGET" --features "$FEATURES"

echo "Define binary and signature paths"
BIN="$APP_PATH/$APP_NAME"
SIG="$OUTPUT_DIR/$APP_NAME-$FEATURES.sig"

echo "Listing target directory:"
ls -l "$APP_PATH"
[ -f "$BIN" ] || { echo "Error: Binary not found at $BIN"; exit 1; }
file "$BIN"
ldd "$BIN"
#chmod +x "$BIN"
#
#echo "Generating signature at $SIG..."
#"$BIN" || { echo "Error: Binary execution failed with code $?"; exit 1; } &
#PID=$!
#for i in $(seq 1 30); do
#  if [ -f "$SIG" ]; then
#    break
#  fi
#  sleep 1
#done
#kill $PID 2>/dev/null || true
#[ -f "$SIG" ] || { echo "Error: SIG file not found at $SIG"; exit 1; }
#echo "Signature generated at: $SIG"

echo "Copying files to $OUTPUT_DIR..."
mkdir -p "$OUTPUT_DIR"
cp "$BIN" "$OUTPUT_DIR/$APP_NAME-$FEATURES" || { echo "Error: Failed to copy $BIN to $OUTPUT_DIR/$APP_NAME-$FEATURES"; exit 1; }
#cp "$SIG" "$OUTPUT_DIR/$APP_NAME-$FEATURES.sig" || { echo "Error: Failed to copy $SIG to $OUTPUT_DIR/$APP_NAME-$FEATURES.sig"; exit 1; }

echo "Listing output directory in container:"
ls -l "$OUTPUT_DIR"