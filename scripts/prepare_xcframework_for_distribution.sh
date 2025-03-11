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

# Check if GitHub CLI is installed
if ! command -v gh &> /dev/null; then
    echo "🔄 GitHub CLI not found, installing..."
    if [ "$(uname)" == "Darwin" ]; then
        # macOS: install with Homebrew
        if ! command -v brew &> /dev/null; then
            echo "❌ Homebrew not found. Please install Homebrew first: https://brew.sh/"
            exit 1
        fi
        brew install gh
    elif [ "$(expr substr $(uname -s) 1 5)" == "Linux" ]; then
        # Linux: install with apt or other package manager depending on distro
        if command -v apt-get &> /dev/null; then
            curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | sudo dd of=/usr/share/keyrings/githubcli-archive-keyring.gpg \
            && sudo chmod go+r /usr/share/keyrings/githubcli-archive-keyring.gpg \
            && echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | sudo tee /etc/apt/sources.list.d/github-cli.list > /dev/null \
            && sudo apt update \
            && sudo apt install gh -y
        else
            echo "❌ Unsupported Linux distribution"
            exit 1
        fi
    else
        echo "❌ Unsupported operating system"
        exit 1
    fi
fi

# Check if already authenticated with GitHub CLI
if ! gh auth status &> /dev/null; then
    echo "🔄 Not authenticated with GitHub CLI"
    echo "If you cloned with SSH, GitHub CLI should use your SSH credentials."
    echo "Otherwise, please run 'gh auth login' manually."
    
    # Try to authenticate with SSH if available
    if [ -f ~/.ssh/id_rsa ] || [ -f ~/.ssh/id_ed25519 ]; then
        echo "🔄 Attempting to authenticate with SSH..."
        gh auth login --with-token < <(echo $(cat ~/.ssh/id_rsa || cat ~/.ssh/id_ed25519))
    else
        echo "❌ Please run 'gh auth login' manually."
        exit 1
    fi
fi

# Create a release tag using the specified version
RELEASE_TAG="$VERSION"

# Check if the tag already exists
if gh release view "$RELEASE_TAG" --repo "$REPO" &> /dev/null; then
    echo "⚠️ Release $RELEASE_TAG already exists"
    
    # Ask for confirmation to continue
    read -p "Do you want to add the asset to the existing release? (y/n): " -n 1 -r
    echo
    
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "❌ Operation cancelled"
        exit 1
    fi
    
    # Upload the asset to the existing release
    echo "📤 Uploading asset to existing release..."
    gh release upload "$RELEASE_TAG" "$ZIP_FILE" "platforms/android/app/build/outputs/aar/app-release.aar" --repo "$REPO" --clobber
else
    # Create a new release
    echo "🆕 Creating new release $RELEASE_TAG..."
    
    # Prepare release notes
    if [ "$VERSION" = "$COMMIT_HASH" ]; then
        TITLE="$COMMIT_HASH"
        NOTES="Release for commit $COMMIT_HASH"
    else
        TITLE="$VERSION"
        NOTES="Release version $VERSION"
    fi
    
    gh release create "$RELEASE_TAG" \
        --repo "$REPO" \
        --title "$TITLE" \
        --notes "$NOTES" \
        "$ZIP_FILE"
        "platforms/android/app/build/outputs/aar/app-release.aar"
fi

# Get the download URL
DOWNLOAD_URL=$(gh release view "$RELEASE_TAG" --repo "$REPO" --json assets --jq ".assets[] | select(.name==\"ChainKitFFI-$VERSION.zip\") | .url")

echo "✅ Release complete!"
echo ""
echo "📋 Release information:"
echo "- Tag: $RELEASE_TAG"
echo "- Download URL: $DOWNLOAD_URL"
echo "- Checksum: $CHECKSUM"

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
echo "🎉 Release process completed!" 