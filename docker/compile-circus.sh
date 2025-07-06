#!/bin/bash
set -e

APP_NAME="QR2M"
TARGET="x86_64-unknown-linux-musl"
FEATURES="dev"
BUILD_PATH="target/$TARGET"
OUTPUT_DIR="$BUILD_PATH/release"
OUTPUT="false"

CIRCUS="/home/QR2M/compile-circus"
LOG_DIR="$CIRCUS/LOG"
STATIC_DIR="$CIRCUS/STATIC"

mkdir -p "$CIRCUS"
mkdir -p "$LOG_DIR"

cd /home/QR2M/compile-circus

echo "Set PKG_CONFIG_PATH"
export PKG_CONFIG_PATH="$STATIC_DIR/lib:$STATIC_DIR/lib/pkgconfig:/usr/lib/pkgconfig:/usr/share/pkgconfig:/usr/lib/x86_64-linux-musl/pkgconfig:/usr/local/lib/pkgconfig"
echo "PKG_CONFIG_PATH=$PKG_CONFIG_PATH"


echo "Set environment variables for build"
export PKG_CONFIG_ALLOW_CROSS=1
export CFLAGS="-I$STATIC_DIR/include -static"
export LDFLAGS="-L$STATIC_DIR/lib -static"
export OPENSSL_DIR=/usr
export OPENSSL_LIB_DIR=/usr/lib
export OPENSSL_INCLUDE_DIR=/usr/include
export OPENSSL_STATIC=1
export RUSTFLAGS="-C target-feature=+crt-static -C link-arg=-L/usr/lib -C link-arg=-L$STATIC_DIR/lib/pkgconfig -C link-arg=-L$STATIC_DIR/lib -C link-arg=-lssl -C link-arg=-lcrypto -C link-arg=-static"
#export RUSTFLAGS="-C target-feature=+crt-static -C link-arg=-L$STATIC_DIR/lib -C link-arg=-static"


fix_pc_file() {
  local target_file="$1"
  echo "Detecting GTK 4 .pc file..."
  local pc_dir
  local gtk_pc_file
  local target_pc="$STATIC_DIR/lib/pkgconfig/$target_file"

  mkdir -p "$STATIC_DIR/lib/pkgconfig"

  local base_name=$(echo "$target_file" | sed -E 's/-([0-9]+)\.pc$//')
  local version=$(echo "$target_file" | sed -E 's/.*-([0-9]+)\.pc$/\1/')

  local file_variations=("$base_name.pc" "${base_name}${version}.pc" "${base_name}${version}-0.pc" "${base_name}-${version}-0.pc" "${base_name}-${version}.pc")

  for pc_dir in $(echo "$PKG_CONFIG_PATH" | tr ':' '\n'); do
    for file in "${file_variations[@]}"; do
      if [ -f "$pc_dir/$file" ]; then
        gtk_pc_file="$pc_dir/$file"
        break
      fi
      done
    [ -n "$gtk_pc_file" ] && break
  done

  if [ -z "$gtk_pc_file" ]; then
    echo "Error: $target_file (variations: ${file_variations[*]}) not found in PKG_CONFIG_PATH"
    exit 1
  fi

  echo "Found $target_file: $gtk_pc_file"

  if [ ! -f "$target_pc" ]; then
      echo "Copying $gtk_pc_file to $target_pc"
      cp "$gtk_pc_file" "$target_pc" || { echo "Error: Failed to copy $gtk_pc_file to $target_pc"; exit 1; }
  else
      echo "$target_file already exists at $target_pc"
  fi

  local lib_name="${base_name}-${version}"
  [ "$target_file" = "appstream.pc" ] && lib_name="appstream"
  [ "$target_file" = "libadwaita-1.pc" ] && lib_name="libadwaita-1"
  if ! grep -q "$STATIC_DIR/lib" "$target_pc"; then
    echo "Updating Libs in $target_pc"
    sed -i "s|Libs:.*|Libs: -L$STATIC_DIR/lib -l$lib_name -lm|" "$target_pc" || { echo "Error: Failed to update Libs in $target_pc"; exit 1; }
  fi

  local include_dir="${base_name}-${version}.0"
  [ "$target_file" = "appstream.pc" ] && include_dir="appstream"
  [ "$target_file" = "libadwaita-1.pc" ] && include_dir="libadwaita-1.0"
  if ! grep -q "$STATIC_DIR/include" "$target_pc"; then
    echo "Updating Cflags in $target_pc"
    sed -i "s|Cflags:.*|Cflags: -I$STATIC_DIR/include/$include_dir -I$STATIC_DIR/include|" "$target_pc" || { echo "Error: Failed to update Cflags in $target_pc"; exit 1; }
  fi

  local pkg_name="${target_file%.pc}"
  if ! pkg-config --modversion "$pkg_name" > /dev/null 2>&1; then
    echo "Error: pkg-config cannot find $pkg_name"
    echo "Content of $target_pc:"
    cat "$target_pc"
    echo "PKG_CONFIG_PATH=$PKG_CONFIG_PATH"
    pkg-config --cflags --libs "$pkg_name" 2>&1 | tee "$LOG_DIR/pkg_config_${pkg_name}.log"
    exit 1
  else
    echo "Verified: $pkg_name found with version $(pkg-config --modversion "$pkg_name")"
  fi
}

echo "Fixing GTK 4 .pc file naming..."
fix_pc_file "gtk-4.pc"
pkg-config --modversion gtk-4 || { echo "Error: gtk-4 not found"; exit 1; }

fix_pc_file "libadwaita-1.pc"
pkg-config --modversion libadwaita-1 || { echo "Error: libadwaita-1 not found"; exit 1; }


echo "Checking pkg-config for dependencies"
for pkg in gtk-4 libadwaita-1; do
  echo "Checking $pkg..."
  pkg-config --modversion "$pkg" 2>&1 | tee -a "$LOG_DIR/pkg_config_check.log" || { echo "Error: $pkg not found"; exit 1; }
done

echo "Listing .pc files in $STATIC_DIR/lib/pkgconfig:"
ls -l "$STATIC_DIR/lib/pkgconfig" | tee -a "$LOG_DIR/pkg_config_list.log"


echo "Cloning project..."
git clone https://github.com/control-owl/QR2M QR2M
cd QR2M

cargo build --release --target "$TARGET" --features "$FEATURES" --locked -vv 2>&1 | tee "$LOG_DIR/qr2m_build.log"
STATUS=$?
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/qr2m_build.log
  echo "CARGO BUILD QR2M FAIL"
  exit 1
fi

cargo test --release --locked - --no-fail-fast --target "$TARGET" --features "$FEATURES" 2>&1 | tee "$LOG_DIR/qr2m_test.log"
STATUS=$?
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/qr2m_test.log
  echo "CARGO TEST QR2M FAIL"
  exit 1
fi

echo "Listing build directory:"
ls -l "$BUILD_PATH/release"

echo "Checking binary:"
export BIN="$BUILD_PATH/release/$APP_NAME"
[ -f "$BIN" ] || { echo "Error: Binary not found at $BIN"; exit 1; }
file "$BIN" 2>&1 | tee "$LOG_DIR/qr2m_file_check.log"
ldd "$BIN" 2>&1 | tee "$LOG_DIR/qr2m_ldd_check.log"
chmod +x "$BIN"

echo "Creating output binary"
if [ "$OUTPUT" = "true" ]; then
  echo "Copying files to $OUTPUT_DIR..."
  mkdir -p "$OUTPUT_DIR"
  cp "$BIN" "$OUTPUT_DIR" || { echo "Error: Failed to copy $BIN to $OUTPUT_DIR"; exit 1; }
  #chown 1001:1001 "$OUTPUT_DIR/$APP_NAME" || { echo "Error: Failed to change ownership of $OUTPUT_DIR/$APP_NAME"; exit 1; }

  echo "Listing output directory:"
  ls -l "$OUTPUT_DIR"
fi