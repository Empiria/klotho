#!/bin/bash
# Klotho installer script
# Usage: curl -fsSL https://raw.githubusercontent.com/Empiria/klotho/main/install.sh | bash

set -e

main() {
    # Configuration
    REPO="Empiria/klotho"
    INSTALL_DIR="${KLOTHO_INSTALL_DIR:-$HOME/.local/bin}"
    BINARY_NAME="klotho"

    echo "Installing Klotho..."
    echo

    # Detect platform
    PLATFORM=$(detect_platform)
    echo "Detected platform: $PLATFORM"

    # Get latest version
    VERSION=$(get_latest_version)
    echo "Latest version: $VERSION"

    # Download and install
    install "$VERSION" "$PLATFORM"

    echo
    echo "Klotho installed successfully to: $INSTALL_DIR/$BINARY_NAME"
    echo

    # Check if install directory is in PATH
    if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
        echo "WARNING: $INSTALL_DIR is not in your PATH"
        echo "Add the following to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
        echo
        echo "    export PATH=\"\$PATH:$INSTALL_DIR\""
        echo
    fi
}

detect_platform() {
    local os arch

    # Detect OS
    case "$(uname -s)" in
        Linux*)
            os="linux"
            ;;
        Darwin*)
            os="macos"
            ;;
        MINGW*|MSYS*|CYGWIN*)
            os="windows"
            ;;
        *)
            echo "Error: Unsupported operating system: $(uname -s)" >&2
            exit 1
            ;;
    esac

    # Detect architecture
    case "$(uname -m)" in
        x86_64|amd64)
            arch="x86_64"
            ;;
        aarch64|arm64)
            arch="aarch64"
            ;;
        *)
            echo "Error: Unsupported architecture: $(uname -m)" >&2
            exit 1
            ;;
    esac

    echo "${os}-${arch}"
}

get_latest_version() {
    # Query GitHub API for latest release
    local api_url="https://api.github.com/repos/$REPO/releases/latest"
    local version

    if command -v curl > /dev/null 2>&1; then
        version=$(curl -fsSL "$api_url" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    elif command -v wget > /dev/null 2>&1; then
        version=$(wget -qO- "$api_url" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    else
        echo "Error: Neither curl nor wget found. Please install one of them." >&2
        exit 1
    fi

    if [ -z "$version" ]; then
        echo "Error: Could not fetch latest version from GitHub" >&2
        exit 1
    fi

    echo "$version"
}

install() {
    local version="$1"
    local platform="$2"
    local binary_suffix=""

    # Windows binary has .exe extension
    if [[ "$platform" == windows-* ]]; then
        binary_suffix=".exe"
    fi

    # Construct download URL
    local asset_name="${BINARY_NAME}-${platform}${binary_suffix}"
    local download_url="https://github.com/$REPO/releases/download/$version/$asset_name"
    local checksum_url="${download_url}.sha256"

    echo "Downloading from: $download_url"

    # Create install directory
    mkdir -p "$INSTALL_DIR"

    # Download binary
    local temp_file=$(mktemp)
    if command -v curl > /dev/null 2>&1; then
        curl -fsSL -o "$temp_file" "$download_url"
    elif command -v wget > /dev/null 2>&1; then
        wget -qO "$temp_file" "$download_url"
    fi

    # Download and verify checksum
    if command -v curl > /dev/null 2>&1; then
        curl -fsSL "$checksum_url" | sed "s|$asset_name|$temp_file|" | shasum -a 256 -c - > /dev/null
    elif command -v wget > /dev/null 2>&1; then
        wget -qO- "$checksum_url" | sed "s|$asset_name|$temp_file|" | shasum -a 256 -c - > /dev/null
    fi

    echo "Checksum verified"

    # Install binary
    local target_path="$INSTALL_DIR/${BINARY_NAME}${binary_suffix}"
    mv "$temp_file" "$target_path"
    chmod +x "$target_path"
}

# Run main function
main "$@"
