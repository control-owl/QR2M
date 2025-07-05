#!/bin/bash
set -e

CIRCUS="/home/QR2M/compile-circus"
LOG_DIR="$CIRCUS/OUTPUT/"

mkdir -p $CIRCUS
mkdir -p $LOG_DIR

cd $CIRCUS

cargo install cargo-c 2>&1 | tee "$LOG_DIR/cargo-c_install.log"
STATUS=$?
if [ "$STATUS" -ne 0 ]; then
  cat $LOG_DIR/cargo-c_install.log
  echo "CARGO INSTALL CARGO-C FAIL"
  exit 1
fi

cargo-cbuild --version

echo "cargo-c installed successfully"