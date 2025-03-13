#!/bin/bash
set -e

REPO="jup-ag/chainkit"

# Use version provided as parameter, or git commit hash if not provided
COMMIT_HASH=$(git rev-parse --short HEAD)
VERSION=${1:-$COMMIT_HASH}

echo "🚀 Creating GitHub release"
echo "📋 Version: $VERSION"
if [ "$VERSION" = "$COMMIT_HASH" ]; then
    echo "ℹ️ Using commit hash as version: $COMMIT_HASH"
fi

# Check if GitHub CLI is installed
if ! command -v gh &> /dev/null; then
    echo "🔄 GitHub CLI not found, installing..."
    if ! command -v brew &> /dev/null; then
        echo "❌ Homebrew not found. Please install Homebrew first: https://brew.sh/"
        exit 1
    fi
    brew install gh
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
    read -p "Do you want to continue with the existing release? (y/n): " -n 1 -r
    echo
    
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "❌ Operation cancelled"
        exit 1
    fi
    
    echo "🔄 Using existing release $RELEASE_TAG"
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
        --notes "$NOTES"
fi

echo "✅ Release created successfully!"
echo "- Tag: $RELEASE_TAG"
echo "- Repository: $REPO"

# Export the release tag for other scripts to use
export RELEASE_TAG="$RELEASE_TAG" 