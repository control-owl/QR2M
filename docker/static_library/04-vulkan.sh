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

export PKG_CONFIG_LIBDIR="/home/QR2M/compile-circus/STATIC/lib/pkgconfig"
export PKG_CONFIG_PATH="/home/QR2M/compile-circus/STATIC/share/pkgconfig:/usr/lib/pkgconfig:/usr/share/pkgconfig:/usr/lib/x86_64-linux-musl/pkgconfig:/usr/local/lib/pkgconfig"
export PKG_CONFIG="pkg-config --static"
export CFLAGS="-I/home/QR2M/compile-circus/STATIC/include -O2 -fno-semantic-interposition -Wno-maybe-uninitialized"
export LDFLAGS="-L/home/QR2M/compile-circus/STATIC/lib -lz -latomic"
#export RUSTFLAGS="-C link-arg=-L/home/QR2M/compile-circus/STATIC/lib -C link-arg=-lz -C link-arg=-latomic"

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  needed_files=(
#    "libcurl.pc"
#    "libxml-2.0.pc"
#    "libeconf.pc"
#    "libunistring.a"
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
  git clone https://github.com/KhronosGroup/Vulkan-Headers.git --depth 1 vulkan-headers
  cd vulkan-headers
  mkdir -p builddir && cd builddir
  cmake \
    -DCMAKE_INSTALL_PREFIX="$STATIC_DIR" \
    -DBUILD_SHARED_LIBS=0 \
    -DCMAKE_BUILD_TYPE=Release \
    ..
  make -j"$(nproc)"
  make install
} 2>&1 | tee -a "$LOG_FILE"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat "$LOG_FILE"
  exit 1
fi

cd "$CIRCUS"

{
  git clone https://github.com/KhronosGroup/Vulkan-Loader.git --depth 1 vulkan-loader
} 2>&1 | tee -a "$LOG_FILE"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat "$LOG_FILE"
  exit 1
fi

cd vulkan-loader

# Patch CMakeLists.txt to enforce static library only
{
  sed -i '/add_library(vulkan/ s/SHARED/STATIC/' CMakeLists.txt
  sed -i '/add_library(vulkan/ s/ SHARED / STATIC /' CMakeLists.txt
  sed -i 's/set_target_properties(vulkan PROPERTIES OUTPUT_NAME vulkan.*$/set_target_properties(vulkan PROPERTIES OUTPUT_NAME vulkan)/' CMakeLists.txt
} 2>&1 | tee -a "$LOG_FILE"

# Patch CMakeLists.txt to disable beta extensions
{
  sed -i 's/target_compile_definitions(loader_common_options INTERFACE VK_ENABLE_BETA_EXTENSIONS)/# Disabled VK_ENABLE_BETA_EXTENSIONS/' CMakeLists.txt
} 2>&1 | tee -a "$LOG_FILE"

# Patch vk_object_types.h to remove CUDA-related object types
{
  sed -i '/VK_OBJECT_TYPE_CUDA_MODULE_NV/d' loader/generated/vk_object_types.h
  sed -i '/VK_OBJECT_TYPE_CUDA_FUNCTION_NV/d' loader/generated/vk_object_types.h
} 2>&1 | tee -a "$LOG_FILE"


{
  mkdir -p builddir
  cd builddir
  cmake \
    -DCMAKE_INSTALL_PREFIX="$STATIC_DIR" \
    -DBUILD_SHARED_LIBS=0 \
    -DCMAKE_BUILD_TYPE=Release \
    ..
  make -j"$(nproc)"
  make install
} 2>&1 | tee -a "$LOG_FILE"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat "$LOG_FILE"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.
#
#{
#  ninja
#} 2>&1 | tee -a "$LOG_FILE"
#
#STATUS=${PIPESTATUS[0]}
#if [ "$STATUS" -ne 0 ]; then
#  cat "$LOG_FILE"
#  exit 1
#fi
#
## -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.
#
#{
#  ninja install
#} 2>&1 | tee -a "$LOG_FILE"
#
#STATUS=${PIPESTATUS[0]}
#if [ "$STATUS" -ne 0 ]; then
#  cat "$LOG_FILE"
#  exit 1
#fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

if [ -f "$STATIC_DIR/lib/libvulkan.so" ]; then
  ar rcs "$STATIC_DIR/lib/libvulkan.a" "$STATIC_DIR/lib/libvulkan.so"
  if [ "$?" -ne 0 ]; then
    echo "Error: Failed to create libvulkan.a from libvulkan.so"
    exit 1
  fi
  echo "Create libvulkan.a from libvulkan.so"
else
  echo "Error: Shared library $STATIC_DIR/lib/libvulkan.so not found"
  if [ -d "$STATIC_DIR/lib" ]; then
    ls -l "$STATIC_DIR/lib/libvulkan*"
  fi
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  compiled_files=(
    "libvulkan.a"
    "vulkan.pc"
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
