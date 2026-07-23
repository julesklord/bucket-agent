#!/usr/bin/env bash
# install.sh — Install bucket-agent from GitHub Releases
# Usage: curl -fsSL https://raw.githubusercontent.com/julesklord/bucket-agent/main/scripts/install.sh | bash

set -Eeuo pipefail

# Configuration
REPO="julesklord/bucket-agent"
BINARY_NAME="bucket"
INSTALL_DIR="${BUCKET_HOME:-$HOME/.bucket}/bin"

# Colors & Formatting (Terminal output)
setup_colors() {
    if [[ -t 1 ]] && command -v tput >/dev/null 2>&1; then
        BOLD="$(tput bold 2>/dev/null || true)"
        DIM="$(tput dim 2>/dev/null || true)"
        CYAN="$(tput setaf 6 2>/dev/null || true)"
        GREEN="$(tput setaf 2 2>/dev/null || true)"
        YELLOW="$(tput setaf 3 2>/dev/null || true)"
        RED="$(tput setaf 1 2>/dev/null || true)"
        MAGENTA="$(tput setaf 5 2>/dev/null || true)"
        RESET="$(tput sgr0 2>/dev/null || true)"
    else
        BOLD=""
        DIM=""
        CYAN=""
        GREEN=""
        YELLOW=""
        RED=""
        MAGENTA=""
        RESET=""
    fi
}

# Official Bucket Braille Logo (matches bucket-tui logo07)
log_banner() {
    cat <<EOF

  ${MAGENTA}${BOLD}⠀⠀⠀⠀⣶⣶⣶⣶⣶⣶⠀⠀⠀⠀
⠀⠀⠀⢰⣿⣿⣿⣿⣿⣿⡆⠀⠀⠀
⠀⠀⠀⢿⣛⣛⢿⡿⣛⣛⡿⠀⠀⠀
⠀⣠⣾⡜⠛⢋⡊⢑⡙⠛⢣⣷⣄⠀
⠀⢿⣿⡇⠀⠸⠇⠸⠇⠀⢸⣿⡿⠀
⠀⠀⠈⠛⠛⠛⠛⠛⠛⠛⠛⠁⠀⠀${RESET}

  ${CYAN}${BOLD}Bucket AI Agent Installer${RESET} ${DIM}(v0.1)${RESET}
  ${DIM}--------------------------------------------------${RESET}
EOF
}

log_info() {
    printf "  ${CYAN}${BOLD}▶${RESET}  %s\n" "$*"
}

log_success() {
    printf "  ${GREEN}${BOLD}✔${RESET}  %s\n" "$*"
}

log_warn() {
    printf "  ${YELLOW}${BOLD}⚠${RESET}  %s\n" "$*"
}

log_error() {
    printf "  ${RED}${BOLD}✖${RESET}  %s\n" "$*" >&2
}

log_verbose() {
    printf "     ${DIM}├─ %s${RESET}\n" "$*"
}

# Trap cleanup for safety
cleanup() {
    local exit_code=$?
    if [[ $exit_code -ne 0 ]]; then
        echo ""
        log_error "Installation failed with exit code ${exit_code}."
        log_warn "If you need assistance, please open an issue at: https://github.com/${REPO}/issues"
    fi
}
trap cleanup EXIT

# Detect OS and Architecture
detect_platform() {
    local os arch
    os="$(uname -s | tr '[:upper:]' '[:lower:]')"
    arch="$(uname -m)"
    
    log_verbose "Host OS detected: ${os}"
    log_verbose "Host Architecture detected: ${arch}"

    case "$os" in
        linux)  os="linux" ;;
        darwin) os="macos" ;;
        *)
            log_error "Unsupported operating system: ${os}"
            exit 1
            ;;
    esac

    case "$arch" in
        x86_64)         arch="x86_64" ;;
        aarch64|arm64)  arch="aarch64" ;;
        *)
            log_error "Unsupported CPU architecture: ${arch}"
            exit 1
            ;;
    esac

    echo "${os}-${arch}"
}

# Download release binary with verbose output
download_release() {
    local platform="$1"
    local version="${2:-}"

    if [[ -z "$version" ]]; then
        log_info "Querying GitHub API for latest release version..."
        version=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null | grep '"tag_name":' | sed -E 's/.*"v?([^"]+)".*/\1/' || true)
        if [[ -z "$version" ]]; then
            version=$(gh release list --repo "$REPO" --limit 1 --exclude-drafts --json tagName --jq '.[0].tagName' 2>/dev/null | sed 's/^v//' || true)
        fi
        if [[ -z "$version" ]]; then
            log_error "Could not determine latest release version for ${REPO}."
            exit 1
        fi
    fi

    log_success "Target release version resolved: ${BOLD}v${version}${RESET}"
    log_info "Preparing target installation directory..."
    log_verbose "Target path: ${INSTALL_DIR}"
    mkdir -p "$INSTALL_DIR"

    local asset_name="bucket-${version}-${platform}"
    local download_url="https://github.com/${REPO}/releases/download/v${version}/${asset_name}"
    local target_binary="${INSTALL_DIR}/${BINARY_NAME}"

    log_info "Downloading binary artifact..."
    log_verbose "URL: ${download_url}"
    log_verbose "Destination: ${target_binary}"

    if curl --fail --location --progress-bar "$download_url" -o "${target_binary}"; then
        log_success "Download completed successfully."
    else
        log_error "Failed to download binary from ${download_url}."
        exit 1
    fi

    log_info "Applying executable permissions..."
    chmod +x "${target_binary}"
    log_success "Binary configured and executable at: ${BOLD}${target_binary}${RESET}"
}

# Configure Shell PATH automatically
setup_path() {
    log_info "Checking environment PATH configuration..."
    
    if [[ ":$PATH:" == *":$INSTALL_DIR:"* ]]; then
        log_success "Installation directory ${INSTALL_DIR} is already in your PATH."
        return 0
    fi

    local shell_rc=""
    if [[ -f "$HOME/.zshrc" ]]; then
        shell_rc="$HOME/.zshrc"
    elif [[ -f "$HOME/.bashrc" ]]; then
        shell_rc="$HOME/.bashrc"
    elif [[ -f "$HOME/.config/fish/config.fish" ]]; then
        shell_rc="$HOME/.config/fish/config.fish"
    fi

    if [[ -n "$shell_rc" ]]; then
        local path_line="export PATH=\"${INSTALL_DIR}:\$PATH\""
        if [[ "$shell_rc" == *"fish"* ]]; then
            path_line="fish_add_path ${INSTALL_DIR}"
        fi

        if ! grep -qF "${INSTALL_DIR}" "$shell_rc" 2>/dev/null; then
            log_info "Adding ${INSTALL_DIR} to your PATH in ${BOLD}${shell_rc}${RESET}..."
            echo "" >> "$shell_rc"
            echo "# bucket-agent CLI" >> "$shell_rc"
            echo "$path_line" >> "$shell_rc"
            log_success "Updated ${shell_rc} successfully."
        else
            log_verbose "PATH entry already exists in ${shell_rc}."
        fi
    else
        log_warn "No supported shell RC file found (.bashrc, .zshrc, config.fish)."
    fi
}

main() {
    setup_colors
    log_banner

    log_info "Detecting system architecture and operating system..."
    local platform
    platform=$(detect_platform)
    log_success "Target platform target identified: ${BOLD}${platform}${RESET}"

    download_release "$platform" "${1:-}"
    setup_path

    cat <<EOF

  ${GREEN}${BOLD}✔ Installation Complete!${RESET}
  ${DIM}--------------------------------------------------${RESET}
  Start the interactive TUI coding agent by running:

    ${MAGENTA}${BOLD}bucket${RESET}

  ${DIM}Note: If 'bucket' is not recognized immediately, restart your terminal or run:${RESET}
    ${CYAN}export PATH="${INSTALL_DIR}:\$PATH"${RESET}

EOF
}

main "$@"