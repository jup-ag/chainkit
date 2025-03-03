#!/bin/bash

# Set the framework path
PACKAGE_FRAMEWORK_PATH="platforms/ios/ChainKit/Sources/ChainKitFFI.xcframework"

# Check if the framework directory exists
echo "Checking Framework Directory..."
if [ ! -d "$PACKAGE_FRAMEWORK_PATH" ]; then
    echo "Error: XCFramework not found at $PACKAGE_FRAMEWORK_PATH"
    echo "Please build the framework first using 'make apple'"
    exit 1
fi

echo "Found XCFramework at $PACKAGE_FRAMEWORK_PATH"

# Find and compress platform directories
echo "Compressing Platform Directories..."

# Find all directories at depth 1 that aren't zip files
platform_dirs=$(find "$PACKAGE_FRAMEWORK_PATH" -type d -depth 1 | grep -v "\.zip")
platform_count=$(echo "$platform_dirs" | wc -l)

if [ -z "$platform_dirs" ]; then
    echo "Error: No platform directories found in the XCFramework"
    exit 1
fi

echo "Found $platform_count platform directories to compress"

for dir in $platform_dirs; do
    if [ -d "$dir" ]; then
        platform=$(basename "$dir")
        zipfile="$PACKAGE_FRAMEWORK_PATH/$platform.zip"
        
        echo "Creating zip file for $platform..."
        
        # Remove existing zip file if it exists
        if [ -f "$zipfile" ]; then
            rm -f "$zipfile"
            echo "Removed existing zip file: $zipfile"
        fi
        
        # Create the zip file
        (cd "$PACKAGE_FRAMEWORK_PATH" && zip -r "$platform.zip" "$platform" > /dev/null)
        
        if [ $? -eq 0 ]; then
            echo "Successfully created $zipfile"
        else
            echo "Failed to create $zipfile"
        fi
    fi
done

echo "Compression Summary:"
zip_files=$(find "$PACKAGE_FRAMEWORK_PATH" -name "*.zip" | wc -l)
echo "Created $zip_files compressed platform files"
echo "XCFramework is now ready for Swift Package Manager distribution" 