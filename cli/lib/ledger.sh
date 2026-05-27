#!/bin/bash
# Bastion TUI - Local Ledger
# Transaction history with search filtering

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

LEDGER_FILE="$(dirname "$SCRIPT_DIR")/../.bastion_history.json"

# ═══════════════════════════════════════════════════════════════════
# Ledger Initialization
# ═══════════════════════════════════════════════════════════════════

init_ledger() {
    if [[ ! -f "$LEDGER_FILE" ]]; then
        echo '{"transactions":[]}' > "$LEDGER_FILE"
    fi
}

# ═══════════════════════════════════════════════════════════════════
# Fetch On-Chain Transactions from Solana
# ═══════════════════════════════════════════════════════════════════

fetch_onchain_transactions() {
    local limit="${1:-10}"
    
    # Get current identity's public key
    local public_key=""
    local key_file=$(get_identity_key_file "$(get_current_identity)")
    if [[ -f "$key_file" ]]; then
        public_key=$(get_pubkey_from_keypair "$key_file")
    fi
    
    if [[ -z "$public_key" ]]; then
        echo "[]"
        return 1
    fi
    
    # Fetch recent signatures from Solana RPC
    local result
    result=$(get_recent_signatures "$public_key" "$limit" 2>/dev/null)
    
    if [[ -z "$result" ]]; then
        echo "[]"
        return 1
    fi
    
    # Parse and format
    echo "$result" | python3 -c "
import sys, json
from datetime import datetime
try:
    data = json.load(sys.stdin)
    sigs = data.get('result', [])
    
    formatted = []
    for s in sigs:
        tx = {
            'hash': s.get('signature', 'N/A'),
            'type': 'transaction',
            'amount': 'N/A',
            'timestamp': datetime.fromtimestamp(s.get('blockTime', 0)).isoformat() if s.get('blockTime') else '',
            'status': 'success' if s.get('err') is None else 'failed',
            'slot': s.get('slot', 0),
            'onchain': True
        }
        formatted.append(tx)
    
    print(json.dumps(formatted))
except Exception as e:
    print('[]')
" 2>/dev/null || echo "[]"
}

list_onchain_transactions() {
    local limit="${1:-10}"
    
    echo -e "${C_BOLD}${C_CYAN}📡 On-Chain Transactions (Live from Solana)${C_RESET}"
    echo ""
    
    local txs
    txs=$(fetch_onchain_transactions "$limit")
    
    if [[ "$txs" == "[]" || -z "$txs" ]]; then
        echo -e "${C_DIM}  No on-chain transactions found or unable to fetch.${C_RESET}"
        return 1
    fi
    
    echo "$txs" | python3 -c "
import sys, json
from datetime import datetime

data = json.load(sys.stdin)

if not data:
    print('  No transactions found.')
else:
    print(f\"  {'Status':<8} {'Type':<12} {'Signature':<48} {'Slot':<10} {'Time':<16}\")
    print('  ' + '─' * 96)
    
    for tx in data[:10]:
        status = '✓' if tx.get('status') == 'success' else '✗'
        type_str = tx.get('type', 'unknown')
        sig = tx.get('hash', '')[:44] + '...'
        slot = str(tx.get('slot', ''))
        
        try:
            ts = datetime.fromisoformat(tx.get('timestamp', ''))
            time_str = ts.strftime('%m/%d %H:%M')
        except:
            time_str = tx.get('timestamp', '')[:16]
        
        print(f\"  {status:<8} {type_str:<12} {sig:<48} {slot:<10} {time_str:<16}\")
" 2>/dev/null
}

# ═══════════════════════════════════════════════════════════════════
# Add Transaction
# ═══════════════════════════════════════════════════════════════════

add_transaction() {
    local tx_hash="$1"
    local type="$2"      # mint, swap, deposit, order
    local amount="$3"
    local status="$4"    # pending, success, failed
    local identity="$5"
    local extra="$6"     # Additional data (JSON)
    
    init_ledger
    
    local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    local id=$(openssl rand -hex 8 2>/dev/null || python3 -c "import os; print(os.urandom(8).hex())")
    
    python3 << PYEOF
import json

ledger_file = "$LEDGER_FILE"

with open(ledger_file, 'r') as f:
    data = json.load(f)

tx = {
    "id": "$id",
    "hash": "$tx_hash",
    "type": "$type",
    "amount": "$amount",
    "timestamp": "$timestamp",
    "status": "$status",
    "identity": "$identity"
}

extra_str = '$extra'
if extra_str and extra_str.strip():
    try:
        tx["extra"] = json.loads(extra_str)
    except:
        pass

data["transactions"].append(tx)

with open(ledger_file, 'w') as f:
    json.dump(data, f, indent=2)
PYEOF
}

# ═══════════════════════════════════════════════════════════════════
# Update Transaction Status
# ═══════════════════════════════════════════════════════════════════

update_transaction_status() {
    local tx_hash="$1"
    local status="$2"
    
    python3 << EOF
import json

with open('$LEDGER_FILE', 'r') as f:
    data = json.load(f)

for tx in data["transactions"]:
    if tx["hash"] == "$tx_hash":
        tx["status"] = "$status"
        break

with open('$LEDGER_FILE', 'w') as f:
    json.dump(data, f, indent=2)
EOF
}

# ═══════════════════════════════════════════════════════════════════
# List Transactions
# ═══════════════════════════════════════════════════════════════════

list_transactions() {
    local filter_type="$1"
    local limit="${2:-20}"
    
    init_ledger
    
    python3 << EOF
import json
from datetime import datetime

with open('$LEDGER_FILE', 'r') as f:
    data = json.load(f)

txs = data.get("transactions", [])

filter_type = "$filter_type"
if filter_type:
    txs = [t for t in txs if t.get("type") == filter_type]

txs = sorted(txs, key=lambda x: x.get("timestamp", ""), reverse=True)[:$limit]

status_icons = {
    "success": "✓",
    "failed": "✗",
    "pending": "◐"
}

type_icons = {
    "mint": "🪙",
    "swap": "🔄",
    "deposit": "💰",
    "order": "📜",
    "withdraw": "📤"
}

if not txs:
    print("No transactions found.")
else:
    print(f"{'Status':<8} {'Type':<10} {'Amount':<15} {'Hash':<20} {'Time':<20}")
    print("─" * 80)
    
    for tx in txs:
        status = status_icons.get(tx.get("status", "pending"), "?")
        type_icon = type_icons.get(tx.get("type", ""), "•")
        type_str = f"{type_icon} {tx.get('type', 'unknown')}"
        amount = tx.get("amount", "0")
        hash_short = tx.get("hash", "")[:16] + "..."
        
        try:
            ts = datetime.fromisoformat(tx.get("timestamp", "").replace("Z", "+00:00"))
            time_str = ts.strftime("%m/%d %H:%M")
        except:
            time_str = tx.get("timestamp", "")[:16]
        
        print(f"{status:<8} {type_str:<10} {amount:<15} {hash_short:<20} {time_str:<20}")
EOF
}

# ═══════════════════════════════════════════════════════════════════
# Interactive History Browser
# ═══════════════════════════════════════════════════════════════════

history_browser() {
    init_ledger
    
    clear_screen
    show_banner
    draw_section "Transaction History"
    
    local tx_lines
    tx_lines=$(python3 << PYEOF
import json

with open('$LEDGER_FILE', 'r') as f:
    data = json.load(f)

for tx in sorted(data.get("transactions", []), key=lambda x: x.get("timestamp", ""), reverse=True):
    status = "✓" if tx.get("status") == "success" else "✗" if tx.get("status") == "failed" else "◐"
    print(f"{status} | {tx.get('type', ''):<8} | {tx.get('amount', ''):<12} | {tx.get('hash', '')[:20]}... | {tx.get('timestamp', '')[:16]}")
PYEOF
)
    
    if [[ -z "$tx_lines" ]]; then
        msg_info "No transactions recorded yet."
        echo ""
        gum input --placeholder "Press Enter to continue..."
        return
    fi
    
    local selected
    selected=$(echo "$tx_lines" | gum filter --placeholder "Search transactions..." --height 15)
    
    if [[ -n "$selected" ]]; then
        local hash=$(echo "$selected" | grep -oP '[a-f0-9]{20}')
        
        echo ""
        draw_section "Transaction Details"
        python3 << PYEOF
import json

with open('$LEDGER_FILE', 'r') as f:
    data = json.load(f)

for tx in data.get("transactions", []):
    if tx.get("hash", "").startswith("$hash"):
        print(f"  Hash:      {tx.get('hash', 'N/A')}")
        print(f"  Type:      {tx.get('type', 'N/A')}")
        print(f"  Amount:    {tx.get('amount', 'N/A')}")
        print(f"  Status:    {tx.get('status', 'N/A')}")
        print(f"  Identity:  {tx.get('identity', 'N/A')}")
        print(f"  Timestamp: {tx.get('timestamp', 'N/A')}")
        break
PYEOF
    fi
    
    echo ""
    gum input --placeholder "Press Enter to continue..."
}

# ═══════════════════════════════════════════════════════════════════
# Ledger Menu
# ═══════════════════════════════════════════════════════════════════

ledger_menu() {
    while true; do
        clear_screen
        show_banner
        draw_section "Transaction Ledger"
        
        list_onchain_transactions 5
        echo ""
        
        echo -e "${C_BOLD}${C_WHITE}📋 Local Transaction History${C_RESET}"
        echo ""
        list_transactions "" 5
        echo ""
        
        local choice
        choice=$(gum choose \
            "View On-Chain Transactions" \
            "Browse Local History" \
            "Filter by Type" \
            "Export to CSV" \
            "← Back to Main Menu")
        
        case "$choice" in
            "View On-Chain Transactions")
                clear_screen
                show_banner
                draw_section "On-Chain Transactions"
                list_onchain_transactions 20
                echo ""
                gum input --placeholder "Press Enter to continue..."
                ;;
            "Browse Local History")
                history_browser
                ;;
            "Filter by Type")
                local type
                type=$(gum choose "mint" "swap" "deposit" "order" "withdraw")
                clear_screen
                draw_section "Transactions: $type"
                list_transactions "$type" 20
                echo ""
                gum input --placeholder "Press Enter to continue..."
                ;;
            "Export to CSV")
                local csv_file="bastion_history_$(date +%Y%m%d).csv"
                python3 << EOF
import json
import csv

with open('$LEDGER_FILE', 'r') as f:
    data = json.load(f)

with open('$csv_file', 'w', newline='') as f:
    writer = csv.writer(f)
    writer.writerow(['Hash', 'Type', 'Amount', 'Status', 'Identity', 'Timestamp'])
    for tx in data.get('transactions', []):
        writer.writerow([
            tx.get('hash', ''),
            tx.get('type', ''),
            tx.get('amount', ''),
            tx.get('status', ''),
            tx.get('identity', ''),
            tx.get('timestamp', '')
        ])
print("Exported to $csv_file")
EOF
                msg_success "Exported to $csv_file"
                sleep 1
                ;;
            "← Back to Main Menu"|"")
                break
                ;;
        esac
    done
}
