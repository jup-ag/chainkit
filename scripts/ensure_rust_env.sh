#!/bin/bash
set -e

# This script ensures that the Rust environment is properly set up
# It either sources the cargo env or installs Rust if not available

# Guard to prevent multiple initializations
if [ "${CHAINKIT_RUST_ENV_INITIALIZED:-}" = "true" ]; then
    # If already initialized, just ensure cargo env is sourced
    if [ -f "$HOME/.cargo/env" ]; then
        source "$HOME/.cargo/env"
    fi
    return 0 2>/dev/null || exit 0
fi

# Define the standard Cargo paths
CARGO_HOME="$HOME/.cargo"
CARGO_BIN="$CARGO_HOME/bin"
RUSTUP_PATH="$CARGO_BIN/rustup"
CARGO_PATH="$CARGO_BIN/cargo"
CARGO_ENV="$CARGO_HOME/env"

export_rust_path() {
    export PATH="$CARGO_BIN:$PATH"
    echo "------> Exported Rust PATH: $CARGO_BIN"
}

# Ensure Rust targets are installed when needed
ensure_android_targets() {
    # If no targets are specified, use default Android targets
    targets=${1:-"aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android"}
    
    echo "------> Checking Android targets: $targets"
    for target in $targets; do
        if ! rustup target list --installed | grep -q "$target"; then
            echo "------> Adding Rust target: $target"
            rustup target add "$target"
        else
            echo "------> Rust target already installed: $target"
        fi
    done
}

# Setup Rust environment
setup_rust_env() {
    # Check if rustup and cargo are already in PATH
    if command -v rustup &> /dev/null && command -v cargo &> /dev/null; then
        echo "------> Rust toolchain found in PATH"
        # Use the one in PATH
        RUSTUP_PATH=$(command -v rustup)
        CARGO_PATH=$(command -v cargo)
        echo "------> Using rustup: $RUSTUP_PATH"
        echo "------> Using cargo: $CARGO_PATH"
        return 0
    fi

    # Check if rustup exists in the standard location
    if [ -f "$RUSTUP_PATH" ] && [ -f "$CARGO_PATH" ]; then
        echo "------> Found Rust at standard location: $CARGO_BIN"
        export_rust_path
        echo "------> Rust toolchain activated successfully"
        return 0
    elif [ -f "$CARGO_ENV" ]; then
        echo "------> Found Rust environment at $CARGO_ENV, sourcing it..."
        source "$CARGO_ENV"
        
        # Verify that rustup and cargo are now available
        if command -v rustup &> /dev/null && command -v cargo &> /dev/null; then
            echo "------> Rust toolchain activated successfully"
            return 0
        fi
    fi

    # If not found, install Rust
    echo "------> Rust toolchain not found, installing..."
    if [ "$(uname)" == "Darwin" ] || [ "$(expr substr $(uname -s) 1 5)" == "Linux" ]; then
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        
        # Source the environment
        if [ -f "$CARGO_ENV" ]; then
            source "$CARGO_ENV"
            echo "------> Rust toolchain installed successfully"
        else
            echo "ERROR: Rust installed but $CARGO_ENV not found"
            exit 1
        fi
    else
        echo "ERROR: Unsupported operating system for automatic Rust installation"
        echo "Please install Rust manually: https://rustup.rs/"
        exit 1
    fi

    # Verify installation
    if ! command -v rustup &> /dev/null || ! command -v cargo &> /dev/null; then
        echo "ERROR: Rust installation failed or PATH not properly set"
        echo "Please install Rust manually: https://rustup.rs/"
        exit 1
    fi

    # Final check and path export
    export_rust_path
    return 0
}

# Run the setup
setup_rust_env

# Source cargo environment to ensure it's available
if [ -f "$CARGO_ENV" ]; then
    source "$CARGO_ENV"
fi 

# Mark as initialized to prevent redundant setup
export CHAINKIT_RUST_ENV_INITIALIZED=true 