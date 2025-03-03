#!/bin/bash
set -e

FRAMEWORK_PATH="platforms/ios/ChainKit/Sources/ChainKitFFI.xcframework"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
FULL_FRAMEWORK_PATH="$REPO_ROOT/$FRAMEWORK_PATH"

echo "📦 Starting framework extraction process"
echo "🔍 Framework path: $FULL_FRAMEWORK_PATH"

# Make sure the framework directory exists
mkdir -p "$FULL_FRAMEWORK_PATH"

# Find all zip files in the framework directory
ZIP_FILES=$(find "$FULL_FRAMEWORK_PATH" -name "*.zip")
ZIP_COUNT=$(echo "$ZIP_FILES" | grep -c . || true)

if [ "$ZIP_COUNT" -eq 0 ]; then
  echo "ℹ️ No zip files found in $FRAMEWORK_PATH"
  exit 0
fi

echo "ℹ️ Found $ZIP_COUNT zip file(s) to check"

# Process each zip file
for ZIP_PATH in $ZIP_FILES; do
  ZIP_FILENAME=$(basename "$ZIP_PATH")
  EXPECTED_DIR_NAME="${ZIP_FILENAME%.zip}"
  TARGET_DIR_PATH="$FULL_FRAMEWORK_PATH/$EXPECTED_DIR_NAME"
  
  echo "🔍 Checking zip: $ZIP_FILENAME"
  echo "🔍 Target directory: $EXPECTED_DIR_NAME"
  
  # Get modification times for comparison
  ZIP_MOD_TIME=$(stat -f "%m" "$ZIP_PATH" 2>/dev/null || echo "0")
  
  # Check if directory exists
  if [ -d "$TARGET_DIR_PATH" ]; then
    # Directory exists, check if zip is newer
    DIR_MOD_TIME=$(stat -f "%m" "$TARGET_DIR_PATH" 2>/dev/null || echo "0")
    
    # Extract if zip is newer than directory
    if [ "$ZIP_MOD_TIME" -gt "$DIR_MOD_TIME" ]; then
      echo "🔄 Zip file is newer than existing directory, re-extracting"
    else
      echo "✅ Directory up to date, skipping extraction: $EXPECTED_DIR_NAME"
      continue
    fi
  else
    echo "🔄 Directory doesn't exist, extracting: $EXPECTED_DIR_NAME"
  fi
  
  # Extract the zip file to the framework directory
  echo "📦 Extracting $ZIP_FILENAME..."
  unzip -o "$ZIP_PATH" -d "$FULL_FRAMEWORK_PATH"
  
  # Make binary executable
  BINARY_PATH="$TARGET_DIR_PATH/ChainKitFFI.framework/ChainKitFFI"
  if [ -f "$BINARY_PATH" ]; then
    chmod +x "$BINARY_PATH"
    echo "✅ Made binary executable: $BINARY_PATH"
  fi
  
  # Update directory modification time to match zip file (for future comparisons)
  touch -r "$ZIP_PATH" "$TARGET_DIR_PATH"
  
  echo "✅ Successfully extracted $ZIP_FILENAME"
done

# Verify extraction was successful
echo "🔍 Verifying extraction..."
for ZIP_PATH in $ZIP_FILES; do
  ZIP_FILENAME=$(basename "$ZIP_PATH")
  EXPECTED_DIR_NAME="${ZIP_FILENAME%.zip}"
  TARGET_DIR_PATH="$FULL_FRAMEWORK_PATH/$EXPECTED_DIR_NAME"
  BINARY_PATH="$TARGET_DIR_PATH/ChainKitFFI.framework/ChainKitFFI"
  
  if [ -f "$BINARY_PATH" ] && [ -x "$BINARY_PATH" ]; then
    echo "✅ Binary exists and is executable: $EXPECTED_DIR_NAME/ChainKitFFI.framework/ChainKitFFI"
  else
    echo "⚠️ Binary missing or not executable: $EXPECTED_DIR_NAME/ChainKitFFI.framework/ChainKitFFI"
    if [ -f "$BINARY_PATH" ]; then
      chmod +x "$BINARY_PATH"
      echo "  ✓ Fixed permissions"
    fi
  fi
done

echo "🎉 Framework extraction complete!"
exit 0 