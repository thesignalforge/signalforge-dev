#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

print_banner() {
    echo -e "${CYAN}"
    echo "  ____  _                   _  __                      "
    echo " / ___|(_) __ _ _ __   __ _| |/ _| ___  _ __ __ _  ___ "
    echo " \\___ \\| |/ _\` | '_ \\ / _\` | | |_ / _ \\| '__/ _\` |/ _ \\"
    echo "  ___) | | (_| | | | | (_| | |  _| (_) | | | (_| |  __/"
    echo " |____/|_|\\__, |_| |_|\\__,_|_|_|  \\___/|_|  \\__, |\\___|"
    echo "          |___/                             |___/      "
    echo -e "                    ${GREEN}Dev Environment Manager${NC}"
    echo ""
}

log_info() {
    echo -e "${CYAN}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_command() {
    command -v "$1" >/dev/null 2>&1
}

detect_os() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        if [ -f /etc/os-release ]; then
            . /etc/os-release
            OS=$ID
            OS_FAMILY="linux"
        fi
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        OS="macos"
        OS_FAMILY="macos"
    elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]]; then
        OS="windows"
        OS_FAMILY="windows"
    else
        log_error "Unsupported operating system: $OSTYPE"
        exit 1
    fi
    log_info "Detected OS: $OS ($OS_FAMILY)"
}

install_docker_linux() {
    log_info "Installing Docker..."

    # Remove old versions
    sudo apt-get remove -y docker docker-engine docker.io containerd runc 2>/dev/null || true

    # Install prerequisites
    sudo apt-get update
    sudo apt-get install -y \
        ca-certificates \
        curl \
        gnupg \
        lsb-release

    # Add Docker's official GPG key
    sudo mkdir -p /etc/apt/keyrings
    curl -fsSL https://download.docker.com/linux/$OS/gpg | sudo gpg --dearmor -o /etc/apt/keyrings/docker.gpg 2>/dev/null || true

    # Set up the repository
    echo \
        "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/$OS \
        $(lsb_release -cs) stable" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null

    # Install Docker Engine
    sudo apt-get update
    sudo apt-get install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin

    # Add current user to docker group
    sudo usermod -aG docker $USER

    # Start Docker service
    sudo systemctl enable docker
    sudo systemctl start docker

    log_success "Docker installed successfully"
    log_warn "You may need to log out and back in for docker group changes to take effect"
}

install_docker_macos() {
    log_info "Installing Docker Desktop for macOS..."

    if check_command brew; then
        brew install --cask docker
        log_success "Docker Desktop installed. Please open Docker Desktop to complete setup."
    else
        log_error "Homebrew not found. Please install Docker Desktop manually from https://docker.com/products/docker-desktop"
        exit 1
    fi
}

install_docker() {
    if check_command docker; then
        log_success "Docker is already installed: $(docker --version)"
        return
    fi

    case $OS_FAMILY in
        linux)
            install_docker_linux
            ;;
        macos)
            install_docker_macos
            ;;
        windows)
            log_error "Please install Docker Desktop manually from https://docker.com/products/docker-desktop"
            exit 1
            ;;
    esac
}

install_rust() {
    if check_command rustc && check_command cargo; then
        log_success "Rust is already installed: $(rustc --version)"
        return
    fi

    log_info "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    log_success "Rust installed successfully: $(rustc --version)"
}

install_nodejs() {
    if check_command node && check_command npm; then
        log_success "Node.js is already installed: $(node --version)"
        return
    fi

    log_info "Installing Node.js..."

    case $OS_FAMILY in
        linux)
            # Install via NodeSource
            curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
            sudo apt-get install -y nodejs
            ;;
        macos)
            if check_command brew; then
                brew install node
            else
                log_error "Homebrew not found. Please install Node.js manually."
                exit 1
            fi
            ;;
        windows)
            log_error "Please install Node.js manually from https://nodejs.org"
            exit 1
            ;;
    esac

    log_success "Node.js installed successfully: $(node --version)"
}

install_tauri_deps_linux() {
    log_info "Installing Tauri system dependencies..."

    sudo apt-get update
    sudo apt-get install -y \
        libwebkit2gtk-4.1-dev \
        libappindicator3-dev \
        librsvg2-dev \
        patchelf \
        libssl-dev \
        libgtk-3-dev \
        libayatana-appindicator3-dev \
        libsoup-3.0-dev \
        libjavascriptcoregtk-4.1-dev \
        build-essential \
        curl \
        wget \
        file

    log_success "Tauri dependencies installed"
}

install_tauri_deps_macos() {
    log_info "Installing Tauri system dependencies..."

    if check_command brew; then
        # macOS generally has what's needed, but ensure Xcode CLI tools
        xcode-select --install 2>/dev/null || true
    fi

    log_success "Tauri dependencies installed"
}

install_tauri_deps() {
    case $OS_FAMILY in
        linux)
            install_tauri_deps_linux
            ;;
        macos)
            install_tauri_deps_macos
            ;;
        windows)
            log_info "Windows: Ensure you have Visual Studio Build Tools installed"
            ;;
    esac
}

install_tauri_cli() {
    if check_command cargo; then
        log_info "Installing Tauri CLI..."
        cargo install tauri-cli --locked 2>/dev/null || cargo install tauri-cli
        log_success "Tauri CLI installed"
    fi
}

setup_project() {
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    cd "$SCRIPT_DIR"

    log_info "Installing npm dependencies..."
    npm install

    log_info "Building Rust dependencies (this may take a while on first run)..."
    cd src-tauri
    cargo build --release
    cd ..

    log_success "Project setup complete!"
}

build_app() {
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    cd "$SCRIPT_DIR"

    log_info "Building Signalforge Dev..."
    npm run tauri build

    log_success "Build complete! Find the installer in src-tauri/target/release/bundle/"
}

run_dev() {
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    cd "$SCRIPT_DIR"

    log_info "Starting development server..."
    npm run tauri dev
}

print_usage() {
    echo "Usage: $0 [command]"
    echo ""
    echo "Commands:"
    echo "  install     Install all dependencies (Docker, Rust, Node.js, Tauri)"
    echo "  setup       Install project dependencies only (npm, cargo)"
    echo "  build       Build the application for production"
    echo "  dev         Run in development mode"
    echo "  help        Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 install  # Full installation from scratch"
    echo "  $0 dev      # Run the app in dev mode"
    echo "  $0 build    # Build production installer"
}

main() {
    print_banner
    detect_os

    case "${1:-install}" in
        install)
            log_info "Starting full installation..."
            install_docker
            install_rust
            install_nodejs
            install_tauri_deps
            install_tauri_cli
            setup_project
            echo ""
            log_success "Installation complete!"
            echo ""
            echo -e "Run ${CYAN}./install.sh dev${NC} to start the development server"
            echo -e "Run ${CYAN}./install.sh build${NC} to build the production app"
            ;;
        setup)
            setup_project
            ;;
        build)
            build_app
            ;;
        dev)
            run_dev
            ;;
        help|--help|-h)
            print_usage
            ;;
        *)
            log_error "Unknown command: $1"
            print_usage
            exit 1
            ;;
    esac
}

main "$@"
