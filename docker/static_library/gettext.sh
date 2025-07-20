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
  git clone https://git.savannah.gnu.org/git/gettext.git --depth 1 gettext
} 2>&1 | tee "$LOG_DIR/gettext-01-clone.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat "$LOG_DIR/gettext-01-clone.log"
  echo "ERROR - gettext - 01/05 - Clone"
  exit 1
fi

cd gettext

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  ./autopull.sh
} 2>&1 | tee "$LOG_DIR/gettext-02-autopull.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/gettext-02-autopull.log
  echo "ERROR - gettext - 02/05 - Clone"
  exit 1
fi


# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  ./autogen.sh
} 2>&1 | tee "$LOG_DIR/gettext-02-autogen.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/gettext-02-autogen.log
  echo "ERROR - gettext - 02/05 - Clone"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  ./configure \
    --disable-shared \
    --enable-static \
    --disable-nls \
    --with-pic \
    --disable-java \
    --disable-csharp \
    --disable-c++ \
    --disable-modula2 \
    --disable-libasprintf \
    --disable-curses \
    --disable-openmp \
    --disable-acl \
    --disable-xattr \
    --with-included-libunistring \
    --with-included-gettext \
    --with-included-libxml \
    --without-libncurses-prefix \
    --without-libtermcap-prefix \
    --without-libxcurses-prefix \
    --without-libcurses-prefix \
    --without-libtextstyle-prefix \
    --without-libsmack \
    --without-selinux \
    --without-emacs \
    --without-git \
    --without-bzip2 \
    --without-xz \
    --disable-dependency-tracking \
    --enable-fast-install \
    --disable-rpath \
    --prefix=$STATIC_DIR
} 2>&1 | tee "$LOG_DIR/gettext-03-configure.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/gettext-03-configure.log
  echo "ERROR - gettext - 03/05 - Configure"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  make -C gettext-runtime -j"$(nproc)"
} 2>&1 | tee "$LOG_DIR/gettext-04-make.log"

STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/gettext-04-make.log
  echo "ERROR - gettext - 04/05 - Compile"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

{
  make -C gettext-runtime install-exec
} 2>&1 | tee "$LOG_DIR/gettext-05-install.log"
STATUS=${PIPESTATUS[0]}
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/gettext-05-install.log
  echo "ERROR - gettext - 05/05 - Install"
  exit 1
fi

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

echo "gettext compiled and installed successfully"
