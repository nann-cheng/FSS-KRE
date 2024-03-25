#!/bin/bash

# Directory to search for binary files
SEARCH_DIR="data/"

# Find binary files and calculate their total size
total_size=0
while IFS= read -r -d '' file; do
  if file "$file" | grep -q -v 'text'; then
    # It's a binary file, add its size
    size=$(wc -c <"$file")
    total_size=$((total_size + size))
  fi
done < <(find "$SEARCH_DIR" -type f -print0)

# real_size=$((total_size / (2*1024*1024)))

echo "Total binary file bytes is: $total_size Bytes"
# echo "Total binary file size in $SEARCH_DIR: $real_size MB"


rm -rf "$SEARCH_DIR"/*
echo "Content cleared."