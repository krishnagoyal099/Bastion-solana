#!/bin/bash
# Bastion TUI - Whale Mode
# Iceberg order execution

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# ═══════════════════════════════════════════════════════════════════
# Iceberg Order Generator
# ═══════════════════════════════════════════════════════════════════

generate_chunks() {
    local total="$1"
    local num_chunks="${2:-5}"
    
    python3 << EOF
import random

total = float('$total')
num_chunks = int('$num_chunks')

weights = [random.uniform(0.5, 1.5) for _ in range(num_chunks)]
total_weight = sum(weights)

chunks = [round(total * w / total_weight, 4) for w in weights]

diff = total - sum(chunks)
chunks[-1] = round(chunks[-1] + diff, 4)

for chunk in chunks:
    print(chunk)
EOF
}

# ═══════════════════════════════════════════════════════════════════
# Iceberg Order Execution
# ═══════════════════════════════════════════════════════════════════

execute_iceberg() {
    local total_amount="$1"
    local side="$2"
    local num_chunks="${3:-5}"
    
    draw_section "ICEBERG ORDER"
    
    echo -e "${C_WHITE}${ICON_WHALE} Breaking ${C_CYAN}$total_amount SOL${C_WHITE} into $num_chunks hidden chunks${C_RESET}"
    echo ""
    
    local chunks
    mapfile -t chunks < <(generate_chunks "$total_amount" "$num_chunks")
    
    echo -e "${C_BOLD}Execution Plan:${C_RESET}"
    echo -e "${C_WHITE}┌──────────┬──────────────┬──────────────┬──────────────┐${C_RESET}"
    printf "${C_WHITE}│${C_RESET} %-8s ${C_WHITE}│${C_RESET} %-12s ${C_WHITE}│${C_RESET} %-12s ${C_WHITE}│${C_RESET} %-12s ${C_WHITE}│${C_RESET}\n" \
        "Chunk" "Amount" "Cumulative" "Progress"
    echo -e "${C_WHITE}├──────────┼──────────────┼──────────────┼──────────────┤${C_RESET}"
    
    local cumulative=0
    for i in "${!chunks[@]}"; do
        cumulative=$(python3 -c "print(round($cumulative + ${chunks[$i]}, 4))")
        local pct=$(python3 -c "print(int($cumulative / $total_amount * 100))")
        printf "${C_WHITE}│${C_RESET} %-8s ${C_WHITE}│${C_RESET} ${C_CYAN}%-12s${C_RESET} ${C_WHITE}│${C_RESET} %-12s ${C_WHITE}│${C_RESET} %-12s ${C_WHITE}│${C_RESET}\n" \
            "#$((i+1))" "${chunks[$i]} SOL" "$cumulative" "$pct%"
    done
    
    echo -e "${C_WHITE}└──────────┴──────────────┴──────────────┴──────────────┘${C_RESET}"
    echo ""
    
    gum confirm "Execute iceberg order?" || return
    
    echo ""
    echo -e "${C_BOLD}Executing...${C_RESET}"
    echo ""
    
    cumulative=0
    for i in "${!chunks[@]}"; do
        local chunk="${chunks[$i]}"
        cumulative=$(python3 -c "print(round($cumulative + $chunk, 4))")
        local pct=$(python3 -c "print(int($cumulative / $total_amount * 100))")
        
        printf "  Chunk #%d: ${C_CYAN}%s SOL${C_RESET} " "$((i+1))" "$chunk"
        
        for j in 1 2 3; do
            printf "."
            sleep 0.3
        done
        
        local delay=$(python3 -c "import random; print(round(random.uniform(1, 3), 1))")
        
        printf " ${C_SUCCESS}${ICON_SUCCESS}${C_RESET}\n"
        
        local filled=$((pct * 40 / 100))
        local empty=$((40 - filled))
        printf "         ${C_CYAN}[%s%s]${C_RESET} ${C_WHITE}%3d%%${C_RESET}\n" \
            "$(printf '█%.0s' $(seq 1 $filled))" \
            "$(printf '░%.0s' $(seq 1 $empty))" \
            "$pct"
        
        if (( i < ${#chunks[@]} - 1 )); then
            printf "         ${C_DIM}Waiting ${delay}s before next chunk...${C_RESET}\n"
            sleep "$delay"
        fi
    done
    
    echo ""
    echo -e "${C_SUCCESS}${C_BOLD}━━━ ICEBERG ORDER COMPLETE ━━━${C_RESET}"
    echo -e "${C_SUCCESS}  Total Executed: ${C_CYAN}$total_amount SOL${C_RESET}"
    echo -e "${C_SUCCESS}  Chunks: $num_chunks${C_RESET}"
    echo -e "${C_SUCCESS}  Market Impact: ${C_CYAN}Minimized${C_RESET}"
    
    add_transaction "iceberg_$(openssl rand -hex 8 2>/dev/null || python3 -c "import os; print(os.urandom(8).hex())")" "order" "$total_amount" "success" "$(get_current_identity)" '{"type":"iceberg","chunks":'"$num_chunks"'}'
}

# ═══════════════════════════════════════════════════════════════════
# Whale Mode Menu
# ═══════════════════════════════════════════════════════════════════

whale_menu() {
    clear_screen
    show_banner
    
    echo -e "${C_WARN}╔════════════════════════════════════════════════════════════════╗${C_RESET}"
    echo -e "${C_WARN}║             [!]  [SIMULATION] WHALE MODE - ICEBERG ORDERS       ║${C_RESET}"
    echo -e "${C_WARN}╚════════════════════════════════════════════════════════════════╝${C_RESET}"
    echo ""
    echo -e "${C_WHITE}Execute large orders with minimal market impact (simulated).${C_RESET}"
    echo -e "${C_DIM}Orders are split into random chunks and executed with delays.${C_RESET}"
    echo -e "${C_DIM}Real execution requires deployed AMM programs with liquidity.${C_RESET}"
    echo ""
    
    local amount
    amount=$(gum input --placeholder "Total order amount (SOL)" --value "100")
    
    local side
    side=$(gum choose "BUY" "SELL")
    
    local chunks
    chunks=$(gum input --placeholder "Number of chunks (3-10)" --value "5")
    
    echo ""
    execute_iceberg "$amount" "$side" "$chunks"
    
    echo ""
    gum input --placeholder "Press Enter to continue..."
}
