#!/bin/bash
# Bastion TUI - MEV Attack Simulation
# Side-by-side comparison: Public AMM vs Bastion

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# ═══════════════════════════════════════════════════════════════════
# MEV Simulation - Public AMM (Attacked)
# ═══════════════════════════════════════════════════════════════════

simulate_public_amm() {
    local amount="$1"
    local price="$2"
    
    echo -e "${C_ERROR}${C_BOLD}"
    echo "╔══════════════════════════════════════════════════════════════╗"
    echo "║              ⚠️  PUBLIC MEMPOOL - UNPROTECTED                 ║"
    echo "╚══════════════════════════════════════════════════════════════╝"
    echo -e "${C_RESET}"
    
    sleep 0.5
    
    local attacked_price=$(python3 -c "print(round($price * 1.06, 2))")
    
    echo -e "${C_WHITE}┌─ MEMPOOL ─────────────────────────────────────────────────────┐${C_RESET}"
    echo -e "${C_WHITE}│${C_RESET}"
    echo -e "${C_WHITE}│${C_RESET}  ${C_CYAN}12:00:00.100${C_RESET} │ ${C_BOLD}[YOUR TX]${C_RESET} Swap ${C_CYAN}$amount SOL${C_RESET} @ \$${price}"
    echo -e "${C_WHITE}│${C_RESET}               │ Direction: ${C_SUCCESS}BUY${C_RESET} │ Status: ${C_YELLOW}PENDING${C_RESET}"
    sleep 0.8
    
    echo -e "${C_WHITE}│${C_RESET}"
    echo -e "${C_WHITE}│${C_RESET}  ${C_CYAN}12:00:00.102${C_RESET} │ ${ICON_BOT} ${C_ERROR}SANDWICH BOT DETECTED YOUR TX!${C_RESET}"
    echo -e "${C_WHITE}│${C_RESET}               │ ${C_DIM}Analyzing: amount=$amount, direction=BUY${C_RESET}"
    sleep 0.6
    
    echo -e "${C_WHITE}│${C_RESET}"
    echo -e "${C_WHITE}│${C_RESET}  ${C_CYAN}12:00:00.103${C_RESET} │ ${ICON_BOT} ${C_ERROR}[BOT FRONT-RUN]${C_RESET} Buy 50 SOL"
    echo -e "${C_WHITE}│${C_RESET}               │ ${C_ERROR}→ Price pushed: \$${price} → \$${attacked_price}${C_RESET}"
    sleep 0.6
    
    local slippage=$(python3 -c "print(round(($attacked_price - $price) / $price * 100, 1))")
    local loss=$(python3 -c "print(round($amount * $attacked_price - $amount * $price, 2))")
    
    echo -e "${C_WHITE}│${C_RESET}"
    echo -e "${C_WHITE}│${C_RESET}  ${C_CYAN}12:00:00.104${C_RESET} │ ${C_BOLD}[YOUR TX]${C_RESET} ${C_WARN}EXECUTED @ \$${attacked_price}${C_RESET}"
    echo -e "${C_WHITE}│${C_RESET}               │ ${C_ERROR}Slippage: -${slippage}%${C_RESET}"
    echo -e "${C_WHITE}│${C_RESET}               │ ${C_ERROR}Value Lost: -\$${loss}${C_RESET}"
    sleep 0.6
    
    echo -e "${C_WHITE}│${C_RESET}"
    echo -e "${C_WHITE}│${C_RESET}  ${C_CYAN}12:00:00.105${C_RESET} │ ${ICON_BOT} ${C_ERROR}[BOT BACK-RUN]${C_RESET} Sell 50 SOL"
    echo -e "${C_WHITE}│${C_RESET}               │ ${C_ERROR}Bot Profit: +\$${loss}${C_RESET}"
    echo -e "${C_WHITE}│${C_RESET}"
    echo -e "${C_WHITE}└────────────────────────────────────────────────────────────────┘${C_RESET}"
    
    echo ""
    echo -e "${C_ERROR}${C_BOLD}  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${C_RESET}"
    echo -e "${C_ERROR}  ${C_BOLD}RESULT: YOU WERE SANDWICHED! Lost \$${loss} to MEV bot ${ICON_BOT}${C_RESET}"
    echo -e "${C_ERROR}${C_BOLD}  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${C_RESET}"
}

# ═══════════════════════════════════════════════════════════════════
# MEV Simulation - Bastion Dark Pool (Protected)
# ═══════════════════════════════════════════════════════════════════

simulate_bastion() {
    local amount="$1"
    local price="$2"
    
    echo -e "${C_SUCCESS}${C_BOLD}"
    echo "╔══════════════════════════════════════════════════════════════╗"
    echo "║              🔐 BASTION ENCRYPTED MEMPOOL                    ║"
    echo "╚══════════════════════════════════════════════════════════════╝"
    echo -e "${C_RESET}"
    
    sleep 0.5
    
    local commitment=$(openssl rand -hex 16 2>/dev/null || python3 -c "import os; print(os.urandom(16).hex())")
    
    echo -e "${C_WHITE}┌─ ENCRYPTED MEMPOOL ───────────────────────────────────────────┐${C_RESET}"
    echo -e "${C_WHITE}│${C_RESET}"
    echo -e "${C_WHITE}│${C_RESET}  ${C_CYAN}12:00:00.100${C_RESET} │ ${C_BOLD}[YOUR TX]${C_RESET} Commitment: ${C_PURPLE}0x${commitment}...${C_RESET}"
    echo -e "${C_WHITE}│${C_RESET}               │ ${C_DIM}ZK Proof: ✓ Valid${C_RESET}"
    sleep 0.8
    
    echo -e "${C_WHITE}│${C_RESET}"
    echo -e "${C_WHITE}│${C_RESET}  ${C_CYAN}12:00:00.102${C_RESET} │ ${ICON_BOT} Bot scanning mempool..."
    sleep 0.4
    echo -e "${C_WHITE}│${C_RESET}               │ ${ICON_HIDDEN} ${C_DIM}Cannot read order amount${C_RESET}"
    sleep 0.3
    echo -e "${C_WHITE}│${C_RESET}               │ ${ICON_HIDDEN} ${C_DIM}Cannot read order direction${C_RESET}"
    sleep 0.3
    echo -e "${C_WHITE}│${C_RESET}               │ ${ICON_HIDDEN} ${C_DIM}Cannot read limit price${C_RESET}"
    sleep 0.3
    echo -e "${C_WHITE}│${C_RESET}               │ ${ICON_BOT} ${C_WARN}Unable to front-run. Skipping.${C_RESET}"
    sleep 0.6
    
    echo -e "${C_WHITE}│${C_RESET}"
    echo -e "${C_WHITE}│${C_RESET}  ${C_CYAN}12:00:00.104${C_RESET} │ ${C_BOLD}[YOUR TX]${C_RESET} ${C_SUCCESS}EXECUTED @ \$${price}${C_RESET}"
    echo -e "${C_WHITE}│${C_RESET}               │ ${C_SUCCESS}Slippage: 0.0%${C_RESET}"
    echo -e "${C_WHITE}│${C_RESET}               │ ${C_SUCCESS}MEV Extracted: \$0.00${C_RESET}"
    echo -e "${C_WHITE}│${C_RESET}"
    echo -e "${C_WHITE}└────────────────────────────────────────────────────────────────┘${C_RESET}"
    
    echo ""
    echo -e "${C_SUCCESS}${C_BOLD}  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${C_RESET}"
    echo -e "${C_SUCCESS}  ${C_BOLD}RESULT: PROTECTED! Zero MEV extraction. Perfect execution ${ICON_LOCK}${C_RESET}"
    echo -e "${C_SUCCESS}${C_BOLD}  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${C_RESET}"
}

# ═══════════════════════════════════════════════════════════════════
# Full MEV Demo
# ═══════════════════════════════════════════════════════════════════

mev_demo() {
    clear_screen
    show_banner

    draw_section "MEV Attack Simulation"

    echo -e "${C_WHITE}This demo shows how sandwich attacks work on public DEXs"
    echo -e "and how Bastion protects your trades using ZK encryption.${C_RESET}"
    echo ""
    echo -e "${C_BOLD}${C_WARN}Real-World MEV Statistics (Solana):${C_RESET}"
    echo -e "  • Solana slot time: ${C_CYAN}~400ms${C_RESET} — MEV bots operate in microseconds"
    echo -e "  • Jito bundle tips: ${C_CYAN}>\$500M+${C_RESET} paid to validators by MEV searchers"
    echo -e "  • Sandwich attacks: ${C_CYAN}~60%${C_RESET} of DEX trades are sandwiched on busy days"
    echo -e "  • Average victim loss: ${C_CYAN}0.5-6%${C_RESET} of trade value per sandwich"
    echo -e "  • ${C_DIM}Source: Jito Labs MEV Dashboard, Flashbots data${C_RESET}"
    echo ""

    local amount
    amount=$(gum input --placeholder "Trade amount (SOL)" --value "10") || true
    [[ -z "$amount" ]] && return

    local price="172.50"

    echo ""
    echo -e "${C_BOLD}Simulation Parameters:${C_RESET}"
    echo -e "  Trade: ${C_CYAN}$amount SOL${C_RESET} (~\$$(python3 -c "print(round(float('$amount') * $price, 2))" 2>/dev/null || echo 0))"
    echo -e "  Price: ${C_CYAN}\$${price}${C_RESET}"
    echo -e "  ${C_DIM}[SIMULATION — no real funds are used]${C_RESET}"
    echo ""

    gum confirm "Run simulation?" || { echo ""; return; }

    echo ""
    echo -e "${C_BOLD}━━━ SCENARIO A: Public AMM (Vulnerable) ━━━${C_RESET}"
    echo ""
    simulate_public_amm "$amount" "$price"

    echo ""
    gum input --placeholder "Press Enter to see Bastion protection..." || true
    echo ""

    echo -e "${C_BOLD}━━━ SCENARIO B: Bastion Dark Pool (Protected) ━━━${C_RESET}"
    echo ""
    simulate_bastion "$amount" "$price"

    echo ""
    gum input --placeholder "Press Enter to continue..." || true
}

# ═══════════════════════════════════════════════════════════════════
# Quick Comparison Table
# ═══════════════════════════════════════════════════════════════════

show_comparison() {
    draw_section "Public DEX vs Bastion Comparison"
    
    echo -e "${C_WHITE}"
    printf "┌──────────────────────┬────────────────────┬────────────────────┐\n"
    printf "│ %-20s │ %-18s │ %-18s │\n" "Feature" "Public DEX" "Bastion"
    printf "├──────────────────────┼────────────────────┼────────────────────┤\n"
    printf "│ %-20s │ ${C_ERROR}%-18s${C_WHITE} │ ${C_SUCCESS}%-18s${C_WHITE} │\n" "Order Visibility" "Fully Visible" "Encrypted"
    printf "│ %-20s │ ${C_ERROR}%-18s${C_WHITE} │ ${C_SUCCESS}%-18s${C_WHITE} │\n" "MEV Exposure" "High Risk" "Protected"
    printf "│ %-20s │ ${C_ERROR}%-18s${C_WHITE} │ ${C_SUCCESS}%-18s${C_WHITE} │\n" "Sandwich Attacks" "Vulnerable" "Impossible"
    printf "│ %-20s │ ${C_WARN}%-18s${C_WHITE} │ ${C_SUCCESS}%-18s${C_WHITE} │\n" "Slippage" "Variable" "Minimal"
    printf "│ %-20s │ ${C_ERROR}%-18s${C_WHITE} │ ${C_SUCCESS}%-18s${C_WHITE} │\n" "Front-running" "Common" "Prevented"
    printf "│ %-20s │ ${C_DIM}%-18s${C_WHITE} │ ${C_SUCCESS}%-18s${C_WHITE} │\n" "Execution Speed" "~400ms" "~400ms"
    printf "└──────────────────────┴────────────────────┴────────────────────┘\n"
    echo -e "${C_RESET}"
}

# ═══════════════════════════════════════════════════════════════════
# Simulation Menu (Entry Point)
# ═══════════════════════════════════════════════════════════════════

simulation_menu() {
    while true; do
        clear_screen
        show_banner
        
        draw_section "Order Flow Simulation"
        
        echo -e "${C_WHITE}Understand how MEV attacks work and how Bastion protects you.${C_RESET}"
        echo ""
        
        local choice
        choice=$(gum choose \
            "Run MEV Attack Demo" \
            "View Comparison Table" \
            "← Back to Main Menu") || true

        [[ -z "$choice" ]] && break

        case "$choice" in
            "Run MEV Attack Demo")
                mev_demo
                ;;
            "View Comparison Table")
                clear_screen
                show_banner
                show_comparison
                echo ""
                gum input --placeholder "Press Enter to continue..." || true
                ;;
            "← Back to Main Menu"|"")
                break
                ;;
        esac
    done
}
