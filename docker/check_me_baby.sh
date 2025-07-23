#!/bin/bash

pc_files=("$@")
log_file="$LOG_DIR/$(basename "${BASH_SOURCE[1]}")-01-verify.log"
missing_files=0

if [ -z "$STATIC_DIR" ]; then
  echo "ERROR: STATIC_DIR is not defined" | tee -a "$log_file"
  exit 1
fi

if [ ! -d "$STATIC_DIR/lib/pkgconfig" ] && [ ! -d "$STATIC_DIR/share/pkgconfig" ]; then
  echo "ERROR: Neither $STATIC_DIR/lib/pkgconfig nor $STATIC_DIR/share/pkgconfig exists" | tee -a "$log_file"
  exit 1
fi

if [ -z "$LOG_DIR" ]; then
  echo "ERROR: LOG_DIR is not defined" | tee -a "$log_file"
  exit 1
fi

if [ ${#pc_files[@]} -eq 0 ]; then
  echo "WARNING: No .pc files specified for checking" | tee -a "$log_file"
  exit 0
fi

echo "Checking .pc files for $(basename "${BASH_SOURCE[1]}"):" | tee -a "$log_file"
for pc in "${pc_files[@]}"; do
  if [ -f "$STATIC_DIR/lib/pkgconfig/$pc" ] || [ -f "$STATIC_DIR/share/pkgconfig/$pc" ]; then
    echo "Found $pc" | tee -a "$log_file"
  else
    echo "ERROR: $pc not found in $STATIC_DIR/lib/pkgconfig or $STATIC_DIR/share/pkgconfig" | tee -a "$log_file"
    missing_files=1
  fi
done

if [ $missing_files -ne 0 ]; then
  echo "ERROR: One or more .pc files are missing for $(basename "${BASH_SOURCE[1]}")" | tee -a "$log_file"
  exit 1
fi

exit 0