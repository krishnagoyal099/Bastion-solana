#!/bin/bash
#
# Bastion CLI Installer
# ZK-Powered Anti-MEV Dark Pool Trading on Solana
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/krishnagoyal099/bastion/main/install-cli.sh | bash
#

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[38;5;141m'
CYAN='\033[38;5;117m'
DIM='\033[2m'
BOLD='\033[1m'
NC='\033[0m'

BASTION_VERSION="1.0.0"
INSTALL_DIR="$HOME/.bastion"
BIN_DIR="$HOME/.local/bin"
REPO_URL="https://github.com/krishnagoyal099/bastion"
RAW_URL="https://raw.githubusercontent.com/krishnagoyal099/bastion/main"

print_banner() {
    echo -e "${PURPLE}"
    cat << 'EOF'
    ____            __  _           
   / __ )____ _____/ /_(_)___  ____ 
  / /_/ / __ `/ __/ __/ / __ \/ __ \
 / /_/ / /_/ (__  ) /_/ / /_/ / / / /
/_____/\__,_/____/\__/_/\____/_/ /_/ 
                                       
    ZK-Powered Dark Pool on Solana
EOF
    echo -e "${NC}"
    echo -e "  ${DIM}v${BASTION_VERSION} • Privacy-Preserving Trading${NC}"
    echo ""
}

log_info() {
    echo -e "  ${BLUE}→${NC} $1"
}

log_success() {
    echo -e "  ${GREEN}✓${NC} $1"
}

log_warn() {
    echo -e "  ${YELLOW}⚠${NC} $1"
}

log_error() {
    echo -e "  ${RED}✗${NC} $1"
    exit 1
}

detect_os() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"
    
    case "$OS" in
        Linux*)     OS_TYPE="linux";;
        Darwin*)    OS_TYPE="macos";;
        *)          log_error "Unsupported operating system: $OS";;
    esac
    
    log_info "Detected: $OS_TYPE ($ARCH)"
}

check_dependencies() {
    log_info "Checking dependencies..."
    
    local missing=()
    
    # Required
    command -v curl >/dev/null 2>&1 || missing+=("curl")
    command -v python3 >/dev/null 2>&1 || missing+=("python3")
    
    # Check for gum
    if ! command -v gum >/dev/null 2>&1; then
        log_warn "gum not found. Installing..."
        install_gum
    else
        log_success "gum $(gum --version 2>/dev/null | head -1 || echo '')"
    fi
    
    # Optional but recommended
    if command -v solana >/dev/null 2>&1; then
        log_success "Solana CLI $(solana --version 2>/dev/null | grep -oP '[\d.]+' | head -1 || echo found)"
    else
        log_warn "Solana CLI not found (optional, recommended for on-chain operations)"
        echo -e "    ${DIM}Install: sh -c \"\$(curl -sSfL https://release.solana.com/stable/install)\"${NC}"
    fi
    
    if [ ${#missing[@]} -ne 0 ]; then
        log_error "Missing required dependencies: ${missing[*]}\nPlease install them and try again."
    fi
    
    log_success "All required dependencies satisfied"
}

install_gum() {
    if [ "$OS_TYPE" = "macos" ]; then
        if command -v brew >/dev/null 2>&1; then
            brew install gum 2>/dev/null
        else
            log_error "Homebrew is required to install gum on macOS.\n  Install: /bin/bash -c \"\$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\""
        fi
    elif [ "$OS_TYPE" = "linux" ]; then
        if command -v apt-get >/dev/null 2>&1; then
            sudo mkdir -p /etc/apt/keyrings
            curl -fsSL https://repo.charm.sh/apt/gpg.key | sudo gpg --dearmor -o /etc/apt/keyrings/charm.gpg 2>/dev/null
            echo "deb [signed-by=/etc/apt/keyrings/charm.gpg] https://repo.charm.sh/apt/ * *" | sudo tee /etc/apt/sources.list.d/charm.list >/dev/null
            sudo apt update -qq && sudo apt install -y -qq gum 2>/dev/null
        elif command -v dnf >/dev/null 2>&1; then
            echo '[charm]
name=Charm
baseurl=https://repo.charm.sh/yum/
enabled=1
gpgcheck=1
gpgkey=https://repo.charm.sh/yum/gpg.key' | sudo tee /etc/yum.repos.d/charm.repo >/dev/null
            sudo dnf install -y -q gum 2>/dev/null
        elif command -v pacman >/dev/null 2>&1; then
            sudo pacman -S --noconfirm gum 2>/dev/null
        else
            log_warn "Could not auto-install gum. Please install manually:"
            echo -e "    ${CYAN}https://github.com/charmbracelet/gum#installation${NC}"
        fi
    fi
}

create_directories() {
    log_info "Creating directories..."
    
    mkdir -p "$INSTALL_DIR"
    mkdir -p "$INSTALL_DIR/cli/lib"
    mkdir -p "$INSTALL_DIR/cli/config"
    mkdir -p "$INSTALL_DIR/keys"
    mkdir -p "$BIN_DIR"
    
    log_success "Directories created"
}

download_files() {
    log_info "Downloading Bastion CLI v${BASTION_VERSION}..."
    
    # Download main CLI entrypoint
    curl -fsSL "$RAW_URL/cli/bastion" -o "$INSTALL_DIR/cli/bastion"
    chmod +x "$INSTALL_DIR/cli/bastion"
    
    # Download CLI modules
    local modules=("ui" "identity" "network" "solana_tx" "ledger" "simulation" "ticker" "liquidity" "arbitrage" "whale" "zkproof")
    
    local failed=0
    for module in "${modules[@]}"; do
        if curl -fsSL "$RAW_URL/cli/lib/${module}.sh" -o "$INSTALL_DIR/cli/lib/${module}.sh" 2>/dev/null; then
            chmod +x "$INSTALL_DIR/cli/lib/${module}.sh"
        else
            log_warn "Failed to download ${module}.sh"
            failed=$((failed + 1))
        fi
    done
    
    # Download config files
    curl -fsSL "$RAW_URL/cli/config/contracts.env" -o "$INSTALL_DIR/cli/config/contracts.env" 2>/dev/null || true
    curl -fsSL "$RAW_URL/cli/config/networks.env" -o "$INSTALL_DIR/cli/config/networks.env" 2>/dev/null || true
    
    if [ $failed -eq 0 ]; then
        log_success "Downloaded all CLI components (${#modules[@]} modules)"
    else
        log_warn "Downloaded with $failed failures (non-critical modules)"
    fi
}

create_launcher() {
    log_info "Creating launcher..."
    
    cat > "$BIN_DIR/bastion" << 'LAUNCHER'
#!/bin/bash
# Bastion CLI Launcher
BASTION_HOME="$HOME/.bastion"
exec "$BASTION_HOME/cli/bastion" "$@"
LAUNCHER
    
    chmod +x "$BIN_DIR/bastion"
    
    log_success "Launcher: $BIN_DIR/bastion"
}

setup_default_network() {
    local network="${1:-devnet}"
    
    # Set default config to devnet for new installs
    cat > "$INSTALL_DIR/cli/config/contracts.env" << EOF
# Bastion Contract Configuration
# Network: $network

CHAIN_NAME=solana-$network
SOLANA_RPC_URL=$([ "$network" = "localnet" ] && echo "http://127.0.0.1:8899" || echo "https://api.$network.solana.com")

# Deployed Program IDs
BASTION_POOL_PROGRAM=CbHS6twCMkYyodaEUtvonRV6HVBZnkGjekohLqXJziU5
BASTION_AMM_PROGRAM=BvFgtfCEeCcMHoN1PRHSXkdzVYTka1NsrVBTeHmnDN2D

# Token Mints
USDC_MINT=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
WSOL_MINT=So11111111111111111111111111111111111111112
EOF
}

update_path() {
    local shell_rc=""
    
    if [ -f "$HOME/.zshrc" ]; then
        shell_rc="$HOME/.zshrc"
    elif [ -f "$HOME/.bashrc" ]; then
        shell_rc="$HOME/.bashrc"
    fi
    
    if [ -n "$shell_rc" ]; then
        if ! grep -q "$BIN_DIR" "$shell_rc" 2>/dev/null; then
            echo "" >> "$shell_rc"
            echo "# Bastion CLI" >> "$shell_rc"
            echo "export PATH=\"\$PATH:$BIN_DIR\"" >> "$shell_rc"
            log_info "Added \$PATH entry to $shell_rc"
        fi
    fi
    
    export PATH="$PATH:$BIN_DIR"
}

print_success() {
    echo ""
    echo -e "${GREEN}${BOLD}  Bastion CLI installed successfully! ${NC}"
    echo ""
    echo -e "  ${BOLD}Quick Start:${NC}"
    echo ""
    echo -e "  ${YELLOW}1.${NC} Reload your shell:"
    echo -e "     ${CYAN}source ~/.bashrc${NC}  ${DIM}(or ~/.zshrc)${NC}"
    echo ""
    echo -e "  ${YELLOW}2.${NC} Launch Bastion:"
    echo -e "     ${CYAN}bastion${NC}"
    echo ""
    echo -e "  ${YELLOW}3.${NC} Switch to devnet for testing:"
    echo -e "     ${CYAN}bastion config set-network devnet${NC}"
    echo ""
    echo -e "  ${BOLD}Other Commands:${NC}"
    echo -e "     ${CYAN}bastion --help${NC}     Show all commands"
    echo -e "     ${CYAN}bastion --version${NC}  Show version"
    echo -e "     ${CYAN}bastion ticker${NC}     Live market ticker"
    echo -e "     ${CYAN}bastion trade${NC}      Quick trade"
    echo ""
    echo -e "  ${DIM}Install dir:  $INSTALL_DIR${NC}"
    echo -e "  ${DIM}Launcher:     $BIN_DIR/bastion${NC}"
    echo ""
}

uninstall() {
    log_info "Uninstalling Bastion..."
    rm -rf "$INSTALL_DIR"
    rm -f "$BIN_DIR/bastion"
    
    # Clean PATH from shell rc
    for rc in "$HOME/.bashrc" "$HOME/.zshrc"; do
        if [ -f "$rc" ]; then
            sed -i '/# Bastion CLI/d' "$rc" 2>/dev/null || true
            sed -i '/\.local\/bin/d' "$rc" 2>/dev/null || true
        fi
    done
    
    log_success "Bastion has been uninstalled"
    exit 0
}

update() {
    log_info "Updating Bastion CLI..."
    download_files
    log_success "Updated to latest version"
    echo -e "  ${DIM}Restart bastion to use the new version${NC}"
    exit 0
}

main() {
    print_banner
    
    case "${1:-}" in
        --uninstall|-u)
            uninstall
            ;;
        --update)
            update
            ;;
    esac
    
    detect_os
    check_dependencies
    create_directories
    download_files
    create_launcher
    setup_default_network "${2:-devnet}"
    update_path
    print_success
}

main "$@"
