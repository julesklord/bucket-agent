#!/usr/bin/env bash
# install.sh — Install bucket-agent from GitHub Releases
# Usage: curl -fsSL https://raw.githubusercontent.com/julesklord/bucket-agent/main/scripts/install.sh | bash

set -euo pipefail

REPO="julesklord/bucket-agent"
BINARY_NAME="bucket"
INSTALL_DIR="${BUCKET_HOME:-$HOME/.bucket}/bin"

# Detect platform
detect_platform() {
    local os arch
    os="$(uname -s | tr '[:upper:]' '[:lower:]')"
    arch="$(uname -m)"
    
    case "$os" in
        linux)  os="linux" ;;
        darwin) os="macos" ;;
        *)      echo "Error: unsupported OS: $os" >&2; exit 1 ;;
    esac
    
    case "$arch" in
        x86_64)  arch="x86_64" ;;
        aarch64|arm64) arch="aarch64" ;;
        *)       echo "Error: unsupported architecture: $arch" >&2; exit 1 ;;
    esac
    
    echo "${os}-${arch}"
}

# Download latest release
download_release() {
    local platform="$1"
    local version="${2:-}"
    
    if [ -z "$version" ]; then
        echo "Fetching latest version..."
        version=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null | grep '"tag_name":' | sed -E 's/.*"v?([^"]+)".*/\1/' || true)
        if [ -z "$version" ]; then
            version=$(gh release list --repo "$REPO" --limit 1 --exclude-drafts --json tagName --jq '.[0].tagName' 2>/dev/null | sed 's/^v//' || true)
        fi
        if [ -z "$version" ]; then
            echo "Error: could not determine latest version" >&2
            exit 1
        fi
    fi
    
    echo "Downloading bucket-agent v${version} for ${platform}..."
    
    local asset_name="bucket-${version}-${platform}"
    local download_url="https://github.com/${REPO}/releases/download/v${version}/${asset_name}"
    
    mkdir -p "$INSTALL_DIR"
    
    curl -fsSL "$download_url" -o "${INSTALL_DIR}/${BINARY_NAME}"
    chmod +x "${INSTALL_DIR}/${BINARY_NAME}"
    
    echo "Installed bucket-agent v${version} to ${INSTALL_DIR}/${BINARY_NAME}"
}

# Add to PATH if needed
setup_path() {
    local shell_rc=""
    
    if [ -f "$HOME/.bashrc" ]; then
        shell_rc="$HOME/.bashrc"
    elif [ -f "$HOME/.zshrc" ]; then
        shell_rc="$HOME/.zshrc"
    fi
    
    if [ -n "$shell_rc" ]; then
        local path_line="export PATH=\"\$HOME/.bucket/bin:\$PATH\""
        if ! grep -qF "$path_line" "$shell_rc" 2>/dev/null; then
            echo "" >> "$shell_rc"
            echo "# bucket-agent" >> "$shell_rc"
            echo "$path_line" >> "$shell_rc"
            echo "Added to PATH in $shell_rc"
        fi
    fi
}

main() {
    local platform
    platform=$(detect_platform)
    
    download_release "$platform" "${1:-}"
    setup_path
    
    echo ""
    echo "Installation complete! Run 'bucket' to start."
    echo "You may need to restart your shell or run:"
    echo "  export PATH=\"\$HOME/.bucket/bin:\$PATH\""
}

main "$@"