#!/bin/bash
set -e

# Set the framework path and repository
FRAMEWORK_PATH="platforms/ios/ChainKit/Sources/ChainKitFFI.xcframework"
FRAMEWORK_DIR=$(dirname "$FRAMEWORK_PATH")
REPO="jup-ag/chainkit"
PACKAGE_SWIFT="Package.swift"

# Use version provided as parameter, or git commit hash if not provided
COMMIT_HASH=$(git rev-parse --short HEAD)
VERSION=${1:-$COMMIT_HASH}

echo "📦 Preparing and uploading XCFramework"
echo "🔍 Framework path: $FRAMEWORK_PATH"
echo "📋 Version: $VERSION"
if [ "$VERSION" = "$COMMIT_HASH" ]; then
    echo "ℹ️ Using commit hash as version: $COMMIT_HASH"
fi

# Check if the framework directory exists
if [ ! -d "$FRAMEWORK_PATH" ]; then
    echo "❌ Error: XCFramework not found at $FRAMEWORK_PATH"
    echo "Please build the framework first using 'make apple'"
    exit 1
fi

# Create the zip file name with version
ZIP_FILE="$FRAMEWORK_DIR/ChainKitFFI-$VERSION.zip"

echo "📦 Creating zip archive of the entire XCFramework..."
# Get just the framework name without the path
FRAMEWORK_NAME=$(basename "$FRAMEWORK_PATH")
# Switch to the directory containing the framework to create a cleaner zip
# This makes the zip contain just the framework instead of the full path
cd "$FRAMEWORK_DIR"
zip -r "ChainKitFFI-$VERSION.zip" "$FRAMEWORK_NAME" -x "*.DS_Store" -x "*.git*" > /dev/null
# Go back to the original directory
cd - > /dev/null

if [ $? -ne 0 ]; then
    echo "❌ Failed to create zip archive"
    exit 1
fi

echo "✅ Successfully created $ZIP_FILE"

# Calculate checksum for Swift Package Manager
CHECKSUM=$(swift package compute-checksum "$ZIP_FILE")
echo "🔑 Checksum: $CHECKSUM"

RELEASE_TAG="$VERSION"

# Get the download URL
DOWNLOAD_URL="https://github.com/$REPO/releases/download/$VERSION/ChainKitFFI-$VERSION.zip"

# Update Package.swift with the new URL and checksum
echo "🔄 Updating Package.swift..."

if [ ! -f "$PACKAGE_SWIFT" ]; then
    echo "⚠️ Package.swift not found. Skipping auto-update."
else
    # Create a backup of the original file
    cp "$PACKAGE_SWIFT" "${PACKAGE_SWIFT}.bak"
    
    # First, check if there's a binaryTarget section in the file
    if grep -q "binaryTarget" "$PACKAGE_SWIFT" && grep -q "url:" "$PACKAGE_SWIFT"; then
        # Use sed to update the URL and checksum
        # Update URL
        sed -i.tmp -E "s#url: \"[^\"]+\"#url: \"$DOWNLOAD_URL\"#g" "$PACKAGE_SWIFT"
        # Update checksum
        sed -i.tmp -E "s#checksum: \"[^\"]+\"#checksum: \"$CHECKSUM\"#g" "$PACKAGE_SWIFT"
        # Remove tmp files created by sed
        rm -f "${PACKAGE_SWIFT}.tmp"
        
        echo "✅ Package.swift updated successfully with new URL and checksum."
    else
        echo "⚠️ No binaryTarget with URL found in Package.swift. Please update manually."
    fi
fi

echo ""
echo "🎉 XCFramework preparation completed!" 