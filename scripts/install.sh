#!/usr/bin/env bash
# install.sh — Install bucket-agent from GitHub Releases or compile from source
# Usage: curl -fsSL https://raw.githubusercontent.com/julesklord/bucket-agent/main/scripts/install.sh | bash [options]

set -Eeuo pipefail

# Configuration
REPO="julesklord/bucket-agent"
BINARY_NAME="bucket"
INSTALL_DIR="${BUCKET_HOME:-$HOME/.bucket}/bin"

# Defaults
MODE="auto" # Options: auto, binary, build
VERSION=""

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

show_help() {
    cat <<EOF
Usage: install.sh [OPTIONS]

Options:
  --binary, -b        Download pre-compiled binary from GitHub Releases (fast)
  --build, -s         Compile from source code using cargo
  --version, -v VER   Specify version tag to install (e.g. 0.1.6 or v0.1.6)
  --help, -h          Show this help message

Examples:
  curl -fsSL .../install.sh | bash -s -- --binary
  curl -fsSL .../install.sh | bash -s -- --build
  curl -fsSL .../install.sh | bash -s -- --version 0.1.6 --build
EOF
    exit 0
}

parse_args() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --binary|-b)
                MODE="binary"
                shift
                ;;
            --build|--source|-s)
                MODE="build"
                shift
                ;;
            --version|-v)
                VERSION="${2:-}"
                if [[ -z "$VERSION" ]]; then
                    log_error "Option $1 requires a version argument"
                    exit 1
                fi
                shift 2
                ;;
            --help|-h)
                show_help
                ;;
            v[0-9]*|[0-9]*)
                # Positional version argument compatibility
                VERSION="$1"
                shift
                ;;
            *)
                log_error "Unknown option: $1"
                show_help
                ;;
        esac
    done
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

resolve_version() {
    if [[ -n "$VERSION" ]]; then
        VERSION="${VERSION#v}" # strip leading v if present
        log_success "Target version specified: ${BOLD}v${VERSION}${RESET}"
        return 0
    fi

    log_info "Querying GitHub API for latest release version..."
    VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null | grep '"tag_name":' | sed -E 's/.*"v?([^"]+)".*/\1/' || true)
    if [[ -z "$VERSION" ]]; then
        VERSION=$(gh release list --repo "$REPO" --limit 1 --exclude-drafts --json tagName --jq '.[0].tagName' 2>/dev/null | sed 's/^v//' || true)
    fi
    if [[ -z "$VERSION" ]]; then
        log_error "Could not determine latest release version for ${REPO}."
        exit 1
    fi
    log_success "Latest release version resolved: ${BOLD}v${VERSION}${RESET}"
}

prompt_install_mode() {
    if [[ "$MODE" != "auto" ]]; then
        return 0
    fi

    # Interactive prompt if stdout is TTY
    if [[ -t 0 ]] && [[ -t 1 ]]; then
        log_info "Select installation method:"
        echo "     ${CYAN}1)${RESET} Pre-compiled binary ${DIM}(Fastest, recommended)${RESET}"
        echo "     ${CYAN}2)${RESET} Compile from source code ${DIM}(Requires Rust toolchain & protoc)${RESET}"
        printf "     ${BOLD}Choice [1-2] (default 1): ${RESET}"
        read -r choice
        case "$choice" in
            2) MODE="build" ;;
            *) MODE="binary" ;;
        esac
    else
        # Default non-interactive mode to binary
        log_verbose "Non-interactive shell detected, defaulting to pre-compiled binary download."
        MODE="binary"
    fi
}

download_release() {
    local platform="$1"

    log_info "Preparing target installation directory..."
    log_verbose "Target path: ${INSTALL_DIR}"
    mkdir -p "$INSTALL_DIR"

    # Fetch available release tags with assets from GitHub API
    local available_tags=()
    if command -v gh >/dev/null 2>&1; then
        mapfile -t available_tags < <(gh release list --repo "$REPO" --limit 10 --exclude-drafts --json tagName --jq '.[].tagName' 2>/dev/null | sed 's/^v//' || true)
    fi

    if [[ ${#available_tags[@]} -eq 0 ]]; then
        local raw_tags
        raw_tags=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases" 2>/dev/null | grep '"tag_name":' | sed -E 's/.*"v?([^"]+)".*/\1/' || true)
        mapfile -t available_tags <<< "$raw_tags"
    fi

    # Ensure targeted version is first in list
    if [[ -n "$VERSION" ]]; then
        available_tags=("$VERSION" "${available_tags[@]}")
    fi

    local target_binary="${INSTALL_DIR}/${BINARY_NAME}"
    local download_success=false
    local installed_ver=""

    # Fallback iteration over available releases until binary download succeeds
    for ver in "${available_tags[@]}"; do
        [[ -z "$ver" ]] && continue
        local asset_name="bucket-${ver}-${platform}"
        local download_url="https://github.com/${REPO}/releases/download/v${ver}/${asset_name}"

        log_info "Attempting to download binary artifact for v${ver} (${platform})..."
        log_verbose "URL: ${download_url}"

        if curl --fail --location --progress-bar "$download_url" -o "${target_binary}"; then
            download_success=true
            installed_ver="$ver"
            log_success "Downloaded pre-compiled binary for ${BOLD}v${ver}${RESET}."
            break
        else
            log_warn "Binary for v${ver} is not yet available or failed to download. Trying next available release..."
        fi
    done

    if [[ "$download_success" != "true" ]]; then
        log_error "No pre-compiled binary could be downloaded for ${platform} across available releases."
        log_info "Falling back to compiling from source..."
        build_from_source
        return 0
    fi

    log_info "Applying executable permissions..."
    chmod +x "${target_binary}"
    log_success "Binary configured and executable at: ${BOLD}${target_binary}${RESET} (v${installed_ver})"
}

ensure_build_dependencies() {
    log_info "Checking build dependencies for Rust compilation..."

    # Check Git
    if ! command -v git >/dev/null 2>&1; then
        log_error "git is required to build from source."
        exit 1
    fi

    # Check Rust / Cargo
    if ! command -v cargo >/dev/null 2>&1; then
        log_warn "Rust toolchain (cargo) not found. Installing via rustup..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain 1.92.0
        # Source cargo environment for current process
        if [[ -f "$HOME/.cargo/env" ]]; then
            # shellcheck source=/dev/null
            source "$HOME/.cargo/env"
        fi
    fi

    log_verbose "Rust version: $(cargo --version 2>/dev/null || echo 'unknown')"

    # Check DotSlash
    if ! command -v dotslash >/dev/null 2>&1; then
        log_info "Installing DotSlash (required for protobuf resolution)..."
        cargo install dotslash
    fi

    # Check protoc
    if ! command -v protoc >/dev/null 2>&1 && [[ ! -x "bin/protoc" ]]; then
        log_info "Checking protoc compiler..."
        if command -v apt-get >/dev/null 2>&1; then
            log_verbose "Installing protobuf-compiler via apt-get..."
            sudo apt-get update -qq && sudo apt-get install -y -qq protobuf-compiler || true
        elif command -v brew >/dev/null 2>&1; then
            log_verbose "Installing protobuf via Homebrew..."
            brew install protobuf || true
        fi
    fi
}

build_from_source() {
    ensure_build_dependencies

    local build_tmp
    build_tmp=$(mktemp -d)
    log_info "Cloning source code from repository..."
    log_verbose "Branch/Tag: v${VERSION}"
    log_verbose "Temp workspace: ${build_tmp}"

    if git clone --depth 1 --branch "v${VERSION}" "https://github.com/${REPO}.git" "${build_tmp}" 2>/dev/null; then
        log_success "Repository cloned successfully."
    else
        log_warn "Tag v${VERSION} not found, falling back to main branch..."
        git clone --depth 1 "https://github.com/${REPO}.git" "${build_tmp}"
    fi

    log_info "Compiling bucket binary in release mode (this may take a few minutes)..."
    (
        cd "${build_tmp}"
        cargo build -p bucket-bin --release
    )

    log_info "Installing compiled binary to target location..."
    mkdir -p "$INSTALL_DIR"
    cp "${build_tmp}/target/release/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
    chmod +x "${INSTALL_DIR}/${BINARY_NAME}"

    rm -rf "${build_tmp}"
    log_success "Source build complete and binary installed to: ${BOLD}${INSTALL_DIR}/${BINARY_NAME}${RESET}"
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
    parse_args "$@"
    log_banner

    resolve_version
    prompt_install_mode

    log_info "Installation mode selected: ${BOLD}${MODE}${RESET}"

    if [[ "$MODE" == "build" ]]; then
        build_from_source
    else
        log_info "Detecting system architecture and operating system..."
        local platform
        platform=$(detect_platform)
        log_success "Target platform identified: ${BOLD}${platform}${RESET}"
        download_release "$platform"
    fi

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