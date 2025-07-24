#!/bin/bash
# authors = ["Control Owl <qr2m[at]r-o0-t[dot]wtf>"]
# license = "CC-BY-NC-ND-4.0  [2023-2025]  Control Owl"
#
# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

set -e
set -x
set -o pipefail

needed_files=("$@")
log_file="$LOG_DIR/$(basename "${BASH_SOURCE[1]}")-check.log"
missing_files=0

# -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

if [ -z "$STATIC_DIR" ]; then
  echo "ERROR: STATIC_DIR is not defined" | tee -a "$log_file"
  exit 1
fi

if [ -z "$LOG_DIR" ]; then
  echo "ERROR: LOG_DIR is not defined" | tee -a "$log_file"
  exit 1
fi

if [ ! -d "$STATIC_DIR/lib/pkgconfig" ] && [ ! -d "$STATIC_DIR/share/pkgconfig" ]; then
  echo "ERROR: Neither $STATIC_DIR/lib/pkgconfig nor $STATIC_DIR/share/pkgconfig exists" | tee -a "$log_file"
  exit 1
fi

if [ ${#needed_files[@]} -eq 0 ]; then
  echo "WARNING: No files specified for checking" | tee -a "$log_file"
  exit 0
fi

echo "Checking files for $(basename "${BASH_SOURCE[1]}"):" | tee -a "$log_file"

declare -A search_paths=(
  [pc]="$STATIC_DIR/lib/pkgconfig $STATIC_DIR/share/pkgconfig"
  [a]="$STATIC_DIR $STATIC_DIR/lib $STATIC_DIR/lib/pkgconfig $STATIC_DIR/shared $STATIC_DIR/include"
  [default]="$STATIC_DIR/bin"
)

find_file() {
  local file="$1"
  local extension="${file##*.}"
  local paths="${search_paths[$extension]:-${search_paths[default]}}"
  
  for dir in $paths; do
    if find "$dir" -name "$file" | grep -q .; then
      echo "Found file: $file in $dir" | tee -a "$log_file"
      return 0
    fi
  done
  
  echo "ERROR: File $file not found in search paths" | tee -a "$log_file"
  return 1
}

for file in "${needed_files[@]}"; do
  if ! find_file "$file"; then
    ((missing_files++))
  fi
done

if [[ $missing_files -ne 0 ]]; then
  echo "ERROR: One or more files are missing for $(basename "${BASH_SOURCE[1]}")" | tee -a "$log_file"
  exit 1
fi

exit 0