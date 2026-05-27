#!/bin/bash
# Bastion TUI - Live Market Ticker
# Non-flickering real-time display with Solana pool data

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Cache for last known good values
LAST_PRICE_DATA=""

# ═══════════════════════════════════════════════════════════════════
# Price Data — tries on-chain first, falls back to simulation
# ═══════════════════════════════════════════════════════════════════

get_price_data() {
    # Try to get real pool reserves from Solana AMM program
    local reserves
    reserves=$(get_pool_reserves 2>/dev/null) || true

    if [[ "$reserves" != "ERROR:"* && -n "$reserves" ]]; then
        local reserve_sol="${reserves%%:*}"
        local reserve_usdc="${reserves##*:}"

        if [[ "$reserve_sol" =~ ^[0-9]+$ ]] && [[ "$reserve_usdc" =~ ^[0-9.]+$ ]] && [[ "$reserve_sol" != "0" ]]; then
            python3 <<PYEOF
import json
reserve_sol = float('$reserve_sol')
reserve_usdc = float('$reserve_usdc')
price = reserve_usdc / reserve_sol if reserve_sol > 0 else 170.25
lp_tokens = int((reserve_sol * reserve_usdc) ** 0.5) if reserve_sol > 0 and reserve_usdc > 0 else 0
print(json.dumps({
    "price": round(price, 4),
    "change_pct": 0.0,
    "reserve_sol": int(reserve_sol),
    "reserve_usdc": round(reserve_usdc, 2),
    "volume_24h": 0,
    "lp_tokens": lp_tokens,
    "fee_pct": 0.3,
    "live": True
}))
PYEOF
            return 0
        fi
    fi

    # Fallback: simulated market data (realistic SOL/USDC movements)
    python3 << 'PYEOF'
import random
import json
import time

# Seed with time for reproducible but varying prices
random.seed(int(time.time()) // 2)  # Changes every 2 seconds

# SOL/USDC base values
base_price = 172.50
base_reserve_sol = 5000
base_reserve_usdc = 862500
base_volume = 1450000

# Add realistic randomness
price = base_price * (1 + random.uniform(-0.015, 0.015))
reserve_sol = int(base_reserve_sol * (1 + random.uniform(-0.03, 0.03)))
reserve_usdc = round(base_reserve_usdc * (1 + random.uniform(-0.03, 0.03)), 2)
volume = int(base_volume * (1 + random.uniform(-0.2, 0.4)))

change_pct = random.uniform(-3.5, 5.0)

print(json.dumps({
    "price": round(price, 4),
    "change_pct": round(change_pct, 2),
    "reserve_sol": reserve_sol,
    "reserve_usdc": reserve_usdc,
    "volume_24h": volume,
    "lp_tokens": int((reserve_sol * reserve_usdc) ** 0.5),
    "fee_pct": 0.3,
    "simulated": True
}))
PYEOF
}

# ═══════════════════════════════════════════════════════════════════
# Render Ticker Display
# ═══════════════════════════════════════════════════════════════════

render_ticker() {
    local data="$1"

    local price change reserve_sol reserve_usdc volume lp_tokens is_sim
    price=$(echo "$data" | python3 -c "import sys,json; print(json.load(sys.stdin)['price'])" 2>/dev/null) || price="0"
    change=$(echo "$data" | python3 -c "import sys,json; print(json.load(sys.stdin)['change_pct'])" 2>/dev/null) || change="0"
    reserve_sol=$(echo "$data" | python3 -c "import sys,json; print(json.load(sys.stdin)['reserve_sol'])" 2>/dev/null) || reserve_sol="0"
    reserve_usdc=$(echo "$data" | python3 -c "import sys,json; print(json.load(sys.stdin)['reserve_usdc'])" 2>/dev/null) || reserve_usdc="0"
    volume=$(echo "$data" | python3 -c "import sys,json; print(json.load(sys.stdin)['volume_24h'])" 2>/dev/null) || volume="0"
    lp_tokens=$(echo "$data" | python3 -c "import sys,json; print(json.load(sys.stdin)['lp_tokens'])" 2>/dev/null) || lp_tokens="0"
    is_sim=$(echo "$data" | python3 -c "import sys,json; d=json.load(sys.stdin); print('SIM' if d.get('simulated') else 'LIVE')" 2>/dev/null) || is_sim="SIM"

    local change_color="$C_SUCCESS"
    local change_arrow="↑"
    local change_neg
    change_neg=$(python3 -c "print(1 if float('$change') < 0 else 0)" 2>/dev/null) || change_neg="0"
    if [[ "$change_neg" == "1" ]]; then
        change_color="$C_ERROR"
        change_arrow="↓"
    fi

    local depth_sol
    depth_sol=$(python3 -c "print(round(float('$reserve_sol') * float('$price'), 2))" 2>/dev/null) || depth_sol="0"

    local data_tag=""
    if [[ "$is_sim" == "SIM" ]]; then
        data_tag="${C_DIM}[simulated]${C_RESET}"
    else
        data_tag="${C_SUCCESS}[LIVE]${C_RESET}"
    fi

    echo -e "${C_WHITE}"
    printf "  ${C_BOLD}SOL/USDC${C_RESET}       ${C_CYAN}\$%.2f${C_WHITE}  ${change_color}%s %.2f%s${C_WHITE}  %b\n" "$price" "$change_arrow" "$change" "%" "$data_tag"
    echo "  ────────────────────────────────────────"
    printf "  Pool A:         ${C_CYAN}%-10s SOL${C_WHITE} (\$%s)\n" "$(printf "%'d" ${reserve_sol} 2>/dev/null || echo ${reserve_sol})" "$(printf "%'d" ${depth_sol%.*} 2>/dev/null || echo ${depth_sol%.*})"
    printf "  Pool B:         ${C_CYAN}%-10s USDC${C_WHITE} (Vol: \$%s)\n" "$(printf "%'.2f" ${reserve_usdc} 2>/dev/null || echo ${reserve_usdc})" "$(printf "%'d" ${volume} 2>/dev/null || echo ${volume})"
    printf "  LP Tokens:      ${C_PURPLE}%-10s${C_WHITE}      (Fee: 0.3%%)\n" "$(printf "%'d" ${lp_tokens} 2>/dev/null || echo ${lp_tokens})"
    echo -e "${C_RESET}"
}

# ═══════════════════════════════════════════════════════════════════
# Mini Sparkline Chart
# ═══════════════════════════════════════════════════════════════════

render_sparkline() {
    local -a prices=("$@")
    local chars=(" " "▂" "▃" "▄" "▅" "▆" "▇" "█")

    local min max
    min=$(printf '%s\n' "${prices[@]}" | sort -n | head -1)
    max=$(printf '%s\n' "${prices[@]}" | sort -n | tail -1)

    echo -ne "  ${C_DIM}Price 1h:${C_RESET} ${C_CYAN}"

    for price in "${prices[@]}"; do
        local is_eq
        is_eq=$(python3 -c "print(1 if abs(float('$max') - float('$min')) < 0.001 else 0)" 2>/dev/null) || is_eq="1"
        if [[ "$is_eq" == "1" ]]; then
            local idx=3
        else
            local idx
            idx=$(python3 -c "val = int((float('$price') - float('$min')) / (float('$max') - float('$min')) * 7); print(max(0, min(7, val)))" 2>/dev/null) || idx=3
        fi
        echo -n "${chars[$idx]}"
    done

    echo -e "${C_RESET}"
}

# ═══════════════════════════════════════════════════════════════════
# Live Ticker Loop
# ═══════════════════════════════════════════════════════════════════

run_ticker() {
    hide_cursor
    trap "show_cursor; return 0" INT TERM

    local price_history=()

    echo -e "${C_DIM}Press Ctrl+C to return to menu${C_RESET}"
    echo ""

    tput sc 2>/dev/null || true

    while true; do
        tput rc 2>/dev/null || true

        local data
        data=$(get_price_data) || true

        if [[ -z "$data" ]]; then
            echo -e "${C_WARN}  Waiting for data...${C_RESET}"
            sleep 2
            continue
        fi

        local current_price
        current_price=$(echo "$data" | python3 -c "import sys,json; print(json.load(sys.stdin)['price'])" 2>/dev/null) || current_price="0"

        price_history+=("$current_price")
        if (( ${#price_history[@]} > 20 )); then
            price_history=("${price_history[@]:1}")
        fi

        render_ticker "$data"

        if (( ${#price_history[@]} >= 5 )); then
            render_sparkline "${price_history[@]}"
        else
            echo ""
        fi

        echo -e "${C_DIM}Last update: $(date '+%H:%M:%S') │ Updates every 2s${C_RESET}"

        tput ed 2>/dev/null || true

        sleep 2
    done
}

# ═══════════════════════════════════════════════════════════════════
# Ticker Entry Point
# ═══════════════════════════════════════════════════════════════════

ticker_menu() {
    clear_screen
    show_banner

    draw_section "Live Market Ticker"

    run_ticker
}
