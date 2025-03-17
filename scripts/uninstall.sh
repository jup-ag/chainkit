#!/bin/bash
# Uninstall script for ChainKit development dependencies

set -e

ANDROID_HOME=${ANDROID_HOME:-$HOME/Library/Android/sdk}

echo "------> Uninstalling development dependencies..."
echo "------> This will remove all JDK, NDK, command line tools, etc. installed for this project"
read -p "Are you sure you want to continue? (y/n) " -n 1 -r
echo ""

if [[ $REPLY =~ ^[Yy]$ ]]; then
    # Remove Android SDK components
    if [ -n "$ANDROID_HOME" ] && [ -d "$ANDROID_HOME" ]; then
        echo "------> Removing Android command line tools..."
        rm -rf "$ANDROID_HOME/cmdline-tools"
        echo "------> Removing Android NDK..."
        rm -rf "$ANDROID_HOME/ndk"
        echo "------> Removing Android platform-tools..."
        rm -rf "$ANDROID_HOME/platform-tools"
        echo "------> Removing Android platforms..."
        rm -rf "$ANDROID_HOME/platforms"
        echo "------> Removing Android build-tools..."
        rm -rf "$ANDROID_HOME/build-tools"
        echo "------> Removing Android licenses..."
        rm -rf "$ANDROID_HOME/licenses"
        echo "------> Removing other Android SDK components..."
        find "$ANDROID_HOME" -mindepth 1 -maxdepth 1 -not -name ".android" -type d -exec rm -rf {} \; 2>/dev/null || true
    fi

    # Remove Rust target components for the project
    echo "------> Removing Rust project-specific targets..."
    rustup target remove aarch64-apple-ios aarch64-apple-ios-sim 2>/dev/null || true
    rustup target remove aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android 2>/dev/null || true

    # Remove cargo-ndk
    echo "------> Removing cargo-ndk..."
    cargo uninstall cargo-ndk 2>/dev/null || true

    # Clean all temp JDK directory if any exist
    echo "------> Cleaning temporary JDK installations..."
    find /var/folders -type d -name "tmp.*" -exec find {} -name "jdk_extract" -exec rm -rf {} \; \; 2>/dev/null || true
    find /tmp -type d -name "tmp.*" -exec find {} -name "jdk_extract" -exec rm -rf {} \; \; 2>/dev/null || true
    find /var/tmp -type d -name "tmp.*" -exec find {} -name "jdk_extract" -exec rm -rf {} \; \; 2>/dev/null || true

    # Ask if user wants to uninstall Temurin JDK
    echo ""
    read -p "Do you want to uninstall Temurin JDK? This may affect other projects. (y/n) " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        if [ "$(uname)" = "Darwin" ]; then
            echo "------> Uninstalling Temurin JDK..."
            if [ -d "/Library/Java/JavaVirtualMachines/temurin-21.jdk" ]; then
                sudo rm -rf "/Library/Java/JavaVirtualMachines/temurin-21.jdk"
            fi
            if [ -d "/Library/Java/JavaVirtualMachines/temurin-23.jdk" ]; then
                sudo rm -rf "/Library/Java/JavaVirtualMachines/temurin-23.jdk"
            fi
            if command -v brew >/dev/null 2>&1; then
                echo "------> Uninstalling Temurin via Homebrew..."
                brew uninstall --cask temurin 2>/dev/null || true
            fi
            echo "------> Removing Homebrew Temurin cask directory..."
            if [ -d "/opt/homebrew/Caskroom/temurin" ]; then
                rm -rf "/opt/homebrew/Caskroom/temurin"
            fi
            echo "✅ Temurin JDK uninstalled"
        else
            echo "------> Unsupported OS for automatic JDK uninstallation"
        fi
    else
        echo "------> Skipping Temurin JDK uninstallation"
    fi

    # Clean build artifacts
    echo "------> Cleaning build artifacts..."
    cd "$(dirname "$0")/.." && make clean

    echo ""
    echo "✅ Uninstall completed successfully!"
    echo ""
    echo "NOTE: If you want to completely remove Rust, you can run:"
    echo "      rustup self uninstall"
    echo ""
    echo "NOTE: If you want to completely remove Homebrew, you can run:"
    echo "      /bin/bash -c \"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/uninstall.sh)\""
    echo ""
    echo "NOTE: The Android SDK directory at $ANDROID_HOME may still contain some files."
    echo "      You can remove the entire directory if you no longer need it:"
    echo "      rm -rf \"$ANDROID_HOME\""
else
    echo "------> Uninstall cancelled"
fi 