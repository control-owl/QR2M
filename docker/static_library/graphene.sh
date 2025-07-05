#!/bin/bash
set -e

CIRCUS="/home/QR2M/compile-circus"
LOG_DIR="$CIRCUS/LOG"
STATIC_DIR="$CIRCUS/STATIC"

mkdir -p $CIRCUS
mkdir -p $LOG_DIR
mkdir -p $STATIC_DIR

cd $CIRCUS

git clone https://github.com/ebassi/graphene.git graphene
cd graphene

meson setup builddir \
  --prefix=$STATIC_DIR \
  --default-library=static \
  -Dintrospection=disabled \
  -Dgtk_doc=false \
  -Dtests=false \
  -Dinstalled_tests=false 2>&1 | tee "$LOG_DIR/graphene_setup.log"

STATUS=$?
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/graphene_setup.log
  echo "MESON SETUP GRAPHENE FAIL"
  exit 1
fi

meson compile -C builddir 2>&1 | tee "$LOG_DIR/graphene_compile.log"
STATUS=$?
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/graphene_compile.log
  echo "MESON COMPILE GRAPHENE FAIL"
  exit 1
fi

meson install -C builddir 2>&1 | tee "$LOG_DIR/graphene_install.log"
STATUS=$?
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/graphene_install.log
  echo "MESON INSTALL GRAPHENE FAIL"
  exit 1
fi

echo "graphene compiled and installed successfully"
