#!/bin/bash

needed_files=("$@")
log_file="$LOG_DIR/$(basename "${BASH_SOURCE[1]}")-check.log"
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

if [ ${#needed_files[@]} -eq 0 ]; then
  echo "WARNING: No files specified for checking" | tee -a "$log_file"
  exit 0
fi

echo "Checking files for $(basename "${BASH_SOURCE[1]}"):" | tee -a "$log_file"

for file in "${needed_files[@]}"; do
  case "${file##*.}" in
    pc)
      if [ -f "$STATIC_DIR/lib/pkgconfig/$file" ] || [ -f "$STATIC_DIR/share/pkgconfig/$file" ]; then
        echo "Found .pc file: $file" | tee -a "$log_file"
      else
        echo "ERROR: .pc file $file not found in $STATIC_DIR/lib/pkgconfig or $STATIC_DIR/share/pkgconfig" | tee -a "$log_file"
        ((missing_files++))
      fi
    ;;

    a)
      local search_dirs=(
        "$STATIC_DIR"
        "$STATIC_DIR/lib"
        "$STATIC_DIR/shared"
        "$STATIC_DIR/include"
      )

      for dir in "${search_dirs[@]}"; do
        if find "$dir" -name "$file" | grep -q . ; then
          echo "Found static library: $file in $dir" | tee -a "$log_file"
          break
        else
          echo "ERROR: Static library $file not found in search paths" | tee -a "$log_file"
          ((missing_files++))
        fi
      done
    ;;

    *)
      echo "ERROR: Unsupported file type for $file" | tee -a "$log_file"
      ((missing_files++))
    ;;
  esac
  done

if [ $missing_files -ne 0 ]; then
  echo "ERROR: One or more files are missing for $(basename "${BASH_SOURCE[1]}")" | tee -a "$log_file"
  exit 1
fi

exit 0