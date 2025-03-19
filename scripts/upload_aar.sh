#!/bin/bash
set -e

REPO="jup-ag/chainkit"
AAR_PATH="platforms/android/chainkit/build/outputs/aar/chainkit-release.aar"

# Use version provided as parameter, or git commit hash if not provided
COMMIT_HASH=$(git rev-parse --short HEAD)
VERSION=${1:-$COMMIT_HASH}

echo "📦 Preparing and uploading Android AAR"
echo "🔍 AAR path: $AAR_PATH"
echo "📋 Version: $VERSION"
if [ "$VERSION" = "$COMMIT_HASH" ]; then
    echo "ℹ️ Using commit hash as version: $COMMIT_HASH"
fi

# Check if the AAR file exists
if [ ! -f "$AAR_PATH" ]; then
    echo "❌ Error: AAR not found at $AAR_PATH"
    echo "Please build the Android library first using 'make android'"
    exit 1
fi

# Check if GitHub CLI is installed
if ! command -v gh &> /dev/null; then
    echo "❌ GitHub CLI is required but not installed."
    exit 1
fi

# Upload the AAR to the release
RELEASE_TAG="$VERSION"

# Upload the AAR to the release
echo "📤 Uploading AAR to release $RELEASE_TAG..."
gh release upload "$RELEASE_TAG" "$AAR_PATH" --repo "$REPO" --clobber

# Check if gradle is available for publishing to GitHub Packages
if command -v ./platforms/android/gradlew &> /dev/null; then
    echo "🚀 Publishing AAR to GitHub Packages..."
    
    # Set GitHub package repository credentials using GitHub CLI token
    GITHUB_TOKEN=$(gh auth token)
    if [ -z "$GITHUB_TOKEN" ]; then
        echo "⚠️ Could not get GitHub token for publishing to GitHub Packages."
        echo "Please ensure you are authenticated with GitHub CLI."
        echo "Skipping GitHub Packages publication."
    else
        # Execute gradle publish task with GitHub token
        cd platforms/android
        # First try to delete the existing version if it exists
        echo "🔄 Checking for existing version..."
        ./gradlew chainkit:deleteReleasePublicationFromGitHubPackagesRepository \
            -PgithubToken="$GITHUB_TOKEN" \
            -PlibraryVersion="$VERSION" || true
        
        # Then publish the new version
        echo "📤 Publishing new version..."
        ./gradlew chainkit:publishReleasePublicationToGitHubPackagesRepository \
            -PgithubToken="$GITHUB_TOKEN" \
            -PlibraryVersion="$VERSION"
        cd -
        echo "✅ Published to GitHub Packages successfully!"
    fi
else
    echo "⚠️ Gradle not found. Skipping GitHub Packages publication."
fi

echo ""
echo "✅ Android AAR distribution completed!" 
echo "- Version: $VERSION"
echo "- Asset: $AAR_PATH" 