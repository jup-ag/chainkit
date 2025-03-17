#!/bin/bash
# Script to find JDK 21+ and set up Rust environment before running make

set -e

echo "------> Looking for JDK 21+ installation..."

# Try to find JDK 21+ using java_home utility on macOS
if [ "$(uname)" = "Darwin" ] && command -v /usr/libexec/java_home >/dev/null 2>&1; then
    if JAVA_21_HOME=$(/usr/libexec/java_home -v 21 2>/dev/null || /usr/libexec/java_home -v 21+ 2>/dev/null || /usr/libexec/java_home -v 23 2>/dev/null); then
        echo "------> Found JDK 21+ at: $JAVA_21_HOME"
        export JAVA_HOME="$JAVA_21_HOME"
        export PATH="$JAVA_HOME/bin:$PATH"
        java_version=$(java -version 2>&1 | head -1)
        echo "------> Java version: $java_version"
    fi
fi

# Extract numeric version from java -version output
get_java_numeric_version() {
    java_ver_line=$(java -version 2>&1 | head -1)
    echo "$java_ver_line" | grep -Eo 'version "([0-9]+)' | cut -d'"' -f2
}

# Try to find JDK by searching common locations
if [ -z "$JAVA_HOME" ] || [ $(get_java_numeric_version) -lt 21 ]; then
    echo "------> Searching for JDK 21+ in common locations..."
    for java_path in \
        /Library/Java/JavaVirtualMachines/temurin*.jdk/Contents/Home/bin/java \
        /Library/Java/JavaVirtualMachines/jdk*.jdk/Contents/Home/bin/java \
        /opt/homebrew/Caskroom/temurin/*/Contents/Home/bin/java \
        /Applications/Eclipse\ Adoptium/jdk*/Contents/Home/bin/java \
        /usr/lib/jvm/java*openjdk*/bin/java \
        /usr/lib/jvm/temurin*jdk*/bin/java \
        $HOME/Library/Java/JavaVirtualMachines/*/Contents/Home/bin/java; do
        if [ -x "$java_path" ]; then
            temp_version=$("$java_path" -version 2>&1 | grep -Eo 'version "([0-9]+)' | cut -d'"' -f2)
            if [ -n "$temp_version" ] && [ "$temp_version" -ge 21 ]; then
                echo "------> Found JDK $temp_version at: $java_path"
                export JAVA_HOME="$(dirname "$(dirname "$java_path")")"
                export PATH="$(dirname "$java_path"):$PATH"
                break
            fi
        fi
    done
fi

# Check if we have Java 21+ now
current_version=$(get_java_numeric_version || echo "unknown")
if [ "$current_version" = "unknown" ]; then
    echo "❌ ERROR: Could not determine Java version."
    exit 1
elif [ "$current_version" -ge 21 ]; then
    java_version=$(java -version 2>&1 | head -1)
    echo "------> Using Java: $java_version"
    echo "------> JAVA_HOME: $JAVA_HOME"
else
    echo "❌ ERROR: Found Java version $current_version, but version 21 or newer is required."
    echo "Please install JDK 21+ manually from https://adoptium.net/temurin/releases/?version=21"
    exit 1
fi

# Check for C compiler and install if needed
echo "------> Checking for C compiler..."
CC_FOUND=false

# Check common compiler locations
if [ -x /usr/bin/cc ]; then
    echo "------> Found C compiler at /usr/bin/cc"
    export CC=/usr/bin/cc
    CC_FOUND=true
elif [ -x /usr/bin/gcc ]; then
    echo "------> Found C compiler at /usr/bin/gcc"
    export CC=/usr/bin/gcc
    CC_FOUND=true
elif [ -x /usr/bin/clang ]; then
    echo "------> Found C compiler at /usr/bin/clang"
    export CC=/usr/bin/clang
    CC_FOUND=true
fi

# If no compiler found, try to install one
if [ "$CC_FOUND" = "false" ]; then
    echo "------> No C compiler found, attempting to install..."
    
    # Detect OS
    if [ "$(uname)" = "Darwin" ]; then
        echo "------> macOS detected, installing Command Line Tools..."
        xcode-select --install || true
        if [ -x /usr/bin/cc ]; then
            echo "------> C compiler installed at /usr/bin/cc"
            export CC=/usr/bin/cc
            CC_FOUND=true
        fi
    elif [ -f /etc/debian_version ]; then
        echo "------> Debian/Ubuntu detected, installing build-essential..."
        sudo apt-get update && sudo apt-get install -y build-essential
        if [ -x /usr/bin/cc ]; then
            echo "------> C compiler installed at /usr/bin/cc"
            export CC=/usr/bin/cc
            CC_FOUND=true
        fi
    elif [ -f /etc/redhat-release ]; then
        echo "------> RHEL/CentOS detected, installing Development Tools..."
        sudo yum -y update && sudo yum -y groupinstall "Development Tools"
        if [ -x /usr/bin/cc ]; then
            echo "------> C compiler installed at /usr/bin/cc"
            export CC=/usr/bin/cc
            CC_FOUND=true
        fi
    else
        echo "------> Unknown OS, cannot install C compiler automatically."
    fi
fi

# Final check if we have a C compiler
if [ "$CC_FOUND" = "false" ]; then
    echo "❌ ERROR: Could not find or install a C compiler."
    echo "Please install a C compiler manually (gcc or clang)."
    exit 1
fi

# Ensure Rust environment is set up properly
echo "------> Setting up Rust environment..."
if [ -f ./scripts/ensure_rust_env.sh ]; then
    source ./scripts/ensure_rust_env.sh
    echo "------> Rust environment set up successfully"
else
    echo "------> Warning: ./scripts/ensure_rust_env.sh not found. Using system Rust if available."
    if [ -d "$HOME/.cargo/bin" ]; then
        export PATH="$HOME/.cargo/bin:$PATH"
        echo "------> Added $HOME/.cargo/bin to PATH"
    fi
fi

# Now run the make command with the C compiler explicitly set
echo "------> Running make with arguments: $@"
CC=${CC:-/usr/bin/cc} make "$@" 