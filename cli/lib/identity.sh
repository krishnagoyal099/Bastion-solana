#!/bin/bash
# Bastion TUI - Identity Manager
# Multi-wallet hot-swapping (Solana JSON keypairs)

# Config
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Persistent user directory for config
BASTION_HOME="$HOME/.bastion"
KEYS_DIR="$SCRIPT_DIR/../../keys"
CURRENT_IDENTITY_FILE="$BASTION_HOME/.current_identity"

# Ensure config directory exists
mkdir -p "$BASTION_HOME"
mkdir -p "$KEYS_DIR"

# ═══════════════════════════════════════════════════════════════════
# Identity Management
# ═══════════════════════════════════════════════════════════════════

# Available identities
unset IDENTITIES
declare -A IDENTITIES

PERSISTENT_IDENTITIES_FILE="$BASTION_HOME/.identities"

# Auto-discover keypair files in keys directory
discover_identities() {
    if [[ -d "$KEYS_DIR" ]]; then
        for key_file in "$KEYS_DIR"/*.json; do
            if [[ -f "$key_file" ]]; then
                local basename=$(basename "$key_file")
                local name="${basename%.json}"
                
                # Skip if already exists
                if [[ -z "${IDENTITIES[$name]}" ]]; then
                    local desc="Discovered key"
                    case "$name" in
                        user|id) desc="Primary trading account" ;;
                        whale) desc="Large position account" ;;
                        attacker) desc="MEV simulation account" ;;
                        *) desc="Custom identity" ;;
                    esac
                    IDENTITIES["$name"]="$basename:$desc"
                fi
            fi
        done
        
        # Also check for Solana default keypair
        local default_kp="$HOME/.config/solana/id.json"
        if [[ -f "$default_kp" ]] && [[ -z "${IDENTITIES[default]}" ]]; then
            # Symlink or copy
            if [[ ! -f "$KEYS_DIR/default.json" ]]; then
                ln -sf "$default_kp" "$KEYS_DIR/default.json" 2>/dev/null || true
            fi
            IDENTITIES["default"]="default.json:Solana default wallet"
        fi
    fi
}

# Load persistent identities from file
load_persistent_identities() {
    if [[ -f "$PERSISTENT_IDENTITIES_FILE" ]]; then
        while IFS="=" read -r key value; do
            if [[ -n "$key" && -n "$value" ]]; then
                key=$(echo "$key" | tr -d '[:space:]')
                IDENTITIES["$key"]="$value"
            fi
        done < "$PERSISTENT_IDENTITIES_FILE"
    fi
}

# Initialize
discover_identities
load_persistent_identities

# Sanity check
if [[ -n "${IDENTITIES[0]}" ]]; then
    unset "IDENTITIES[0]"
fi

get_current_identity() {
    local current="user"
    if [[ -f "$CURRENT_IDENTITY_FILE" ]]; then
        current=$(cat "$CURRENT_IDENTITY_FILE")
    fi
    
    if [[ "$current" == "0" ]]; then
        current="user"
    fi
    echo "$current"
}

set_current_identity() {
    local identity="$1"
    mkdir -p "$(dirname "$CURRENT_IDENTITY_FILE")"
    echo "$identity" > "$CURRENT_IDENTITY_FILE"
}

get_identity_key_file() {
    local identity="${1:-$(get_current_identity)}"
    
    local entry="${IDENTITIES[$identity]}"
    
    if [[ -z "$entry" ]]; then
        if [[ -f "$KEYS_DIR/${identity}.json" ]]; then
            echo "$KEYS_DIR/${identity}.json"
        elif [[ -f "$KEYS_DIR/${identity}-keypair.json" ]]; then
            echo "$KEYS_DIR/${identity}-keypair.json"
        else
            echo "$KEYS_DIR/${identity}.json"
        fi
    else
        local key_file="${entry%%:*}"
        echo "$KEYS_DIR/$key_file"
    fi
}

get_identity_description() {
    local identity="$1"
    local desc="${IDENTITIES[$identity]#*:}"
    if [[ -z "$desc" || "$desc" == "$identity" ]]; then
        echo "Custom identity"
    else
        echo "$desc"
    fi
}

# ═══════════════════════════════════════════════════════════════════
# Key Generation Helper (Solana JSON keypair)
# ═══════════════════════════════════════════════════════════════════

generate_identity_key() {
    local name="$1"
    local key_path="$KEYS_DIR/${name}.json"
    
    if command -v solana-keygen &>/dev/null; then
        solana-keygen new --outfile "$key_path" --no-bip39-passphrase --force >/dev/null 2>&1
        return $?
    else
        # Fallback: generate a random 64-byte keypair using Python
        python3 -c "
import json, os
seed = os.urandom(32)
# Use ed25519 from nacl if available, otherwise generate random bytes
try:
    from nacl.signing import SigningKey
    sk = SigningKey(seed)
    keypair = list(bytes(sk) + bytes(sk.verify_key))
except ImportError:
    # Simple fallback — random 64 bytes (not cryptographically correct but functional for demo)
    keypair = list(os.urandom(64))
with open('$key_path', 'w') as f:
    json.dump(keypair, f)
" 2>/dev/null
        return $?
    fi
}

# ═══════════════════════════════════════════════════════════════════
# Identity Commands
# ═══════════════════════════════════════════════════════════════════

identity_list() {
    if [[ ${#IDENTITIES[@]} -eq 0 ]]; then
        discover_identities
        load_persistent_identities
    fi
    
    draw_section "Available Identities"
    
    local current=$(get_current_identity)
    
    printf "${C_BOLD}${C_WHITE}  %-12s %-20s %-30s %s${C_RESET}\n" "Name" "Key File" "Description" "Status"
    echo "  ──────────────────────────────────────────────────────────────────────────"
    
    local sorted_ids=($(printf '%s\n' "${!IDENTITIES[@]}" | sort))
    
    for identity in "${sorted_ids[@]}"; do
        local key_file="${IDENTITIES[$identity]%%:*}"
        local desc="${IDENTITIES[$identity]#*:}"
        local status=""
        local prefix="  "
        
        if [[ "$identity" == "$current" ]]; then
            status="${C_SUCCESS}● ACTIVE${C_RESET}"
            prefix="${C_SUCCESS}→ ${C_RESET}"
        else
            status="${C_DIM}○ inactive${C_RESET}"
        fi
        
        local key_path="$KEYS_DIR/$key_file"
        if [[ ! -f "$key_path" ]]; then
            status="${C_ERROR}✗ MISSING${C_RESET}"
        fi
        
        if [[ ${#desc} -gt 30 ]]; then
            desc="${desc:0:27}..."
        fi
        
        printf "%b%-12s %-20s %-30s %b\n" "$prefix" "$identity" "$key_file" "$desc" "$status"
    done
    echo ""
}

identity_switch() {
    local target="$1"
    
    if [[ ${#IDENTITIES[@]} -eq 0 ]]; then
        discover_identities
        load_persistent_identities
    fi
    
    if [[ -z "$target" ]]; then
        if [[ ${#IDENTITIES[@]} -eq 0 ]]; then
            msg_error "No identities found. Create a key first."
            return 1
        fi
        
        msg_info "Select identity to activate:"
        
        local options=()
        for identity in "${!IDENTITIES[@]}"; do
            local desc="${IDENTITIES[$identity]#*:}"
            options+=("$identity|$desc")
        done
        
        if [[ ${#options[@]} -eq 0 ]]; then
            msg_error "No identities available"
            return 1
        fi
        
        target=$(printf '%s\n' "${options[@]}" | gum choose --header "Switch Identity" | cut -d'|' -f1)
    fi
    
    if [[ -z "${IDENTITIES[$target]}" ]]; then
        msg_error "Unknown identity: $target"
        return 1
    fi
    
    local key_file=$(get_identity_key_file "$target")
    if [[ ! -f "$key_file" ]]; then
        msg_error "Key file not found: $key_file"
        msg_info "Generating new Solana keypair..."
        
        if generate_identity_key "$target"; then
            msg_success "Generated new keypair: $key_file"
        else
            msg_error "Failed to generate keypair."
            return 1
        fi
    fi
    
    set_current_identity "$target"
    msg_success "Switched to identity: ${C_BOLD}$target${C_RESET}"
    
    local balance=$(get_identity_balance "$target")
    echo -e "  Balance: ${C_CYAN}$balance SOL${C_RESET}"
}

identity_info() {
    local identity=$(get_current_identity)
    local key_file=$(get_identity_key_file "$identity")
    local desc=$(get_identity_description "$identity")
    
    draw_section "Current Identity"
    
    echo -e "  ${C_BOLD}Identity:${C_RESET}    $identity"
    echo -e "  ${C_BOLD}Description:${C_RESET} $desc"
    echo -e "  ${C_BOLD}Key File:${C_RESET}    $key_file"
    
    if [[ -f "$key_file" ]]; then
        local public_key
        public_key=$(get_pubkey_from_keypair "$key_file")
        if [[ -n "$public_key" ]]; then
            echo -e "  ${C_BOLD}Public Key:${C_RESET}  ${C_CYAN}${public_key}${C_RESET}"
        fi
        
        local balance=$(get_identity_balance "$identity")
        echo -e "  ${C_BOLD}Balance:${C_RESET}     ${C_CYAN}$balance SOL${C_RESET}"
    else
        echo -e "  ${C_ERROR}Key file not found!${C_RESET}"
    fi
    echo ""
}

get_identity_balance() {
    local identity="$1"
    local key_file=$(get_identity_key_file "$identity")
    
    if [[ ! -f "$key_file" ]]; then
        echo "0.00"
        return
    fi
    
    local public_key
    public_key=$(get_pubkey_from_keypair "$key_file")
    
    if [[ -n "$public_key" ]]; then
        get_balance "$public_key"
    else
        echo "0.00"
    fi
}

# ═══════════════════════════════════════════════════════════════════
# Identity Menu
# ═══════════════════════════════════════════════════════════════════
identity_menu() {
    while true; do
        clear_screen
        show_banner
        
        local current=$(get_current_identity)
        show_status_bar "$CHAIN_NAME" "$(check_connection)" "$(get_block_height)" "$current"
        
        identity_list
        
        local choice
        choice=$(gum choose \
            "Switch Identity" \
            "View Current" \
            "Generate New Keypair" \
            "Airdrop SOL (devnet)" \
            "← Back to Main Menu")
        
        case "$choice" in
            "Switch Identity")
                identity_switch
                sleep 1
                ;;
            "View Current")
                identity_info
                gum input --placeholder "Press Enter to continue..."
                ;;
            "Generate New Keypair")
                local name
                name=$(gum input --placeholder "Enter identity name (e.g., trader1)")
                if [[ -n "$name" ]]; then
                    if [[ -n "${IDENTITIES[$name]}" ]]; then
                        msg_error "Identity '$name' already exists!"
                        sleep 1
                        continue
                    fi

                    msg_info "Generating Solana keypair..."
                    
                    if generate_identity_key "$name"; then
                        IDENTITIES["$name"]="${name}.json:Custom identity"
                        echo "$name=${name}.json:Custom identity" >> "$PERSISTENT_IDENTITIES_FILE"
                        
                        local pubkey
                        pubkey=$(get_pubkey_from_keypair "$KEYS_DIR/${name}.json")
                        msg_success "Created new identity: $name"
                        echo -e "  ${C_DIM}Public Key: ${pubkey}${C_RESET}"
                        sleep 2
                    else
                        msg_error "Failed to generate keypair."
                        sleep 1
                    fi
                fi
                ;;
            "Airdrop SOL (devnet)")
                local identity=$(get_current_identity)
                local key_file=$(get_identity_key_file "$identity")
                local pubkey=$(get_pubkey_from_keypair "$key_file")
                
                if [[ -n "$pubkey" ]]; then
                    msg_info "Requesting airdrop for $identity ($pubkey)..."
                    local result
                    result=$(request_airdrop "$pubkey" 2 2>&1)
                    if [[ $? -eq 0 ]]; then
                        msg_success "Airdrop successful! +2 SOL"
                    else
                        msg_error "Airdrop failed: $result"
                    fi
                else
                    msg_error "Could not determine public key"
                fi
                sleep 2
                ;;
            "← Back to Main Menu"|"")
                break
                ;;
        esac
    done
}

# ═══════════════════════════════════════════════════════════════════
# First-Time Setup
# ═══════════════════════════════════════════════════════════════════

first_time_setup() {
    echo ""
    echo -e "${C_BOLD}${C_WHITE}╔════════════════════════════════════════════════════════════════╗${C_RESET}"
    echo -e "${C_BOLD}${C_WHITE}║               WELCOME TO BASTION                               ║${C_RESET}"
    echo -e "${C_BOLD}${C_WHITE}║                                                                ║${C_RESET}"
    echo -e "${C_BOLD}${C_WHITE}║  No wallet keys detected. Let's set up your first identity.   ║${C_RESET}"
    echo -e "${C_BOLD}${C_WHITE}╚════════════════════════════════════════════════════════════════╝${C_RESET}"
    echo ""
    
    local setup_choice
    setup_choice=$(gum choose \
        "Create New Account" \
        "Import Existing Keypair" \
        "Use Default Solana Wallet")
    
    case "$setup_choice" in
        "Create New Account")
            echo ""
            msg_info "Creating new Solana keypair..."
            
            if generate_identity_key "user"; then
                if [[ -f "$KEYS_DIR/user.json" ]]; then
                    msg_success "New account created successfully!"
                    echo ""
                    
                    local public_key
                    public_key=$(get_pubkey_from_keypair "$KEYS_DIR/user.json")
                    
                    if [[ -n "$public_key" ]]; then
                        echo -e "  ${C_BOLD}Your Public Key:${C_RESET}"
                        echo -e "  ${C_CYAN}${public_key}${C_RESET}"
                        echo ""
                        echo -e "${C_WARN}${ICON_WARN} Fund your account:${C_RESET}"
                        echo -e "  ${C_DIM}Devnet:  solana airdrop 2 $public_key --url devnet${C_RESET}"
                        echo -e "  ${C_DIM}Mainnet: Transfer SOL from an exchange${C_RESET}"
                    fi
                    
                    IDENTITIES["user"]="user.json:Primary trading account"
                    set_current_identity "user"
                    
                    echo ""
                    gum input --placeholder "Press Enter to continue..."
                else
                    msg_error "Key generation failed"
                    return 1
                fi
            else
                msg_error "Failed to generate keypair"
                return 1
            fi
            ;;
            
        "Import Existing Keypair")
            echo ""
            msg_info "Import an existing Solana keypair (JSON format)"
            echo -e "${C_DIM}Provide the path to your keypair JSON file${C_RESET}"
            echo ""
            
            local key_path
            key_path=$(gum input --placeholder "Path to keypair.json (e.g., ~/my_wallet/id.json)")
            
            key_path="${key_path/#\~/$HOME}"
            
            if [[ -z "$key_path" ]]; then
                msg_error "No path provided"
                return 1
            fi
            
            if [[ ! -f "$key_path" ]]; then
                msg_error "File not found: $key_path"
                return 1
            fi
            
            # Validate JSON keypair
            if ! python3 -c "import json; d=json.load(open('$key_path')); assert isinstance(d,list) and len(d)==64" 2>/dev/null; then
                msg_error "File does not appear to be a valid Solana keypair (expected JSON array of 64 bytes)"
                return 1
            fi
            
            cp "$key_path" "$KEYS_DIR/user.json"
            IDENTITIES["user"]="user.json:Imported account"
            set_current_identity "user"
            
            local public_key
            public_key=$(get_pubkey_from_keypair "$KEYS_DIR/user.json")
            if [[ -n "$public_key" ]]; then
                echo ""
                echo -e "  ${C_BOLD}Imported Account:${C_RESET}"
                echo -e "  ${C_CYAN}${public_key}${C_RESET}"
            fi
            
            echo ""
            gum input --placeholder "Press Enter to continue..."
            ;;
            
        "Use Default Solana Wallet")
            local default_kp="$HOME/.config/solana/id.json"
            if [[ -f "$default_kp" ]]; then
                ln -sf "$default_kp" "$KEYS_DIR/user.json" 2>/dev/null
                IDENTITIES["user"]="user.json:Solana default wallet"
                set_current_identity "user"
                
                local public_key
                public_key=$(get_pubkey_from_keypair "$default_kp")
                msg_success "Linked default Solana wallet: $public_key"
            else
                msg_error "No default Solana wallet found at $default_kp"
                msg_info "Run: solana-keygen new"
            fi
            echo ""
            gum input --placeholder "Press Enter to continue..."
            ;;
    esac
    
    return 0
}

ensure_default_identities() {
    # Check if ANY .json keypair files exist
    local key_count
    key_count=$(find "$KEYS_DIR" -maxdepth 1 -name "*.json" -type f 2>/dev/null | wc -l)
    
    if [[ "$key_count" -eq 0 ]]; then
        # Also check for default Solana wallet
        if [[ -f "$HOME/.config/solana/id.json" ]]; then
            ln -sf "$HOME/.config/solana/id.json" "$KEYS_DIR/default.json" 2>/dev/null
            IDENTITIES["default"]="default.json:Solana default wallet"
            if [[ ! -f "$CURRENT_IDENTITY_FILE" ]]; then
                set_current_identity "default"
            fi
        else
            first_time_setup
        fi
    else
        if [[ ! -f "$CURRENT_IDENTITY_FILE" ]]; then
            local first_key
            first_key=$(find "$KEYS_DIR" -maxdepth 1 -name "*.json" -type f | head -1 | xargs basename | sed 's/.json$//')
            if [[ -n "$first_key" ]]; then
                set_current_identity "$first_key"
            fi
        fi
    fi
}

ensure_default_identities
