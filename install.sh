#!/bin/sh
# Dough installer script
# This script downloads and installs the latest version of dough

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

# Configuration
REPO_OWNER="paullj"
REPO_NAME="dough"
BINARY_NAME="dough"
INSTALL_DIR="${HOME}/.local/bin"

# Functions
error() {
    echo "${RED}Error: $1${NC}" >&2
    exit 1
}

info() {
    echo "${GREEN}$1${NC}"
}

warn() {
    echo "${YELLOW}$1${NC}"
}

# Detect OS and architecture
detect_platform() {
    OS=$(uname -s)
    ARCH=$(uname -m)

    case "$OS" in
        Darwin)
            case "$ARCH" in
                arm64)
                    PLATFORM="aarch64-apple-darwin"
                    ;;
                x86_64)
                    error "Intel Mac not currently supported. Please build from source."
                    ;;
                *)
                    error "Unsupported macOS architecture: $ARCH"
                    ;;
            esac
            ;;
        Linux)
            error "Linux not currently supported. Please build from source."
            ;;
        *)
            error "Unsupported OS: $OS"
            ;;
    esac
}

# Get latest release URL
get_latest_release_url() {
    API_URL="https://api.github.com/repos/${REPO_OWNER}/${REPO_NAME}/releases/latest"

    # Try to get the download URL for our platform
    DOWNLOAD_URL=$(curl -s "$API_URL" | \
        grep "browser_download_url.*${PLATFORM}.*tar.gz" | \
        head -n 1 | \
        cut -d '"' -f 4)

    if [ -z "$DOWNLOAD_URL" ]; then
        error "Could not find release for platform: $PLATFORM"
    fi
}

# Main installation
main() {
    info "Installing dough..."

    # Detect platform
    detect_platform
    info "Detected platform: $PLATFORM"

    # Get latest release
    info "Finding latest release..."
    get_latest_release_url

    # Create install directory if it doesn't exist
    mkdir -p "$INSTALL_DIR"

    # Download and extract
    info "Downloading from: $DOWNLOAD_URL"
    TEMP_DIR=$(mktemp -d)
    trap "rm -rf $TEMP_DIR" EXIT

    cd "$TEMP_DIR"
    curl -L -o archive.tar.gz "$DOWNLOAD_URL"
    tar -xzf archive.tar.gz

    # Find and install the binary
    if [ -f "$BINARY_NAME" ]; then
        mv "$BINARY_NAME" "$INSTALL_DIR/"
        chmod +x "$INSTALL_DIR/$BINARY_NAME"
    elif [ -f "$PLATFORM/$BINARY_NAME" ]; then
        mv "$PLATFORM/$BINARY_NAME" "$INSTALL_DIR/"
        chmod +x "$INSTALL_DIR/$BINARY_NAME"
    else
        error "Binary not found in archive"
    fi

    info "Successfully installed to: $INSTALL_DIR/$BINARY_NAME"

    # Check if install dir is in PATH
    case ":$PATH:" in
        *":$INSTALL_DIR:"*)
            info "Installation complete! You can now run: dough"
            ;;
        *)
            warn "Add $INSTALL_DIR to your PATH to use dough globally:"
            echo ""
            echo "  export PATH=\"\$PATH:$INSTALL_DIR\""
            echo ""
            echo "Add this to your shell configuration file (.bashrc, .zshrc, etc.)"
            echo ""
            echo "Or run directly: $INSTALL_DIR/dough"
            ;;
    esac

    # Show version
    "$INSTALL_DIR/$BINARY_NAME" --version
}

# Run main function
main "$@"