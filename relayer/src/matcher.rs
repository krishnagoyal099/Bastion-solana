use crate::encryption::OrderDetails;
use solana_sdk::pubkey::Pubkey;

#[derive(Clone, Debug)]
pub struct ActiveOrder {
    pub commitment: [u8; 32],
    pub beneficiary: Pubkey,
    pub details: OrderDetails,
}

#[derive(Clone, Debug)]
pub struct MatchResult {
    pub buy: ActiveOrder,
    pub sell: ActiveOrder,
    pub execution_price: u64,
    pub matched_amount: u64,
}

pub struct MatchingEngine {
    // Buy orders sorted by price descending (highest first)
    // Sell orders sorted by price ascending (lowest first)
    pub buys: Vec<ActiveOrder>,
    pub sells: Vec<ActiveOrder>,
}

impl MatchingEngine {
    pub fn new() -> Self {
        Self {
            buys: Vec::new(),
            sells: Vec::new(),
        }
    }

    pub fn add_order(&mut self, commitment: [u8; 32], beneficiary: Pubkey, details: OrderDetails) {
        let order = ActiveOrder { commitment, beneficiary, details };
        if order.details.side == 0 { // 0 = Buy
            self.buys.push(order);
            self.buys.sort_by(|a, b| b.details.price.cmp(&a.details.price));
        } else {
            self.sells.push(order);
            self.sells.sort_by(|a, b| a.details.price.cmp(&b.details.price));
        }
    }

    /// F15 FIX: Match with partial fills
    /// Returns matched orders. If one side has a larger amount, the remainder
    /// is re-inserted into the book as a residual order.
    pub fn match_orders(&mut self) -> Option<MatchResult> {
        if self.buys.is_empty() || self.sells.is_empty() {
            return None;
        }

        let best_buy = &self.buys[0];
        let best_sell = &self.sells[0];

        if best_buy.details.price >= best_sell.details.price {
            let execution_price = best_sell.details.price;

            let buy_amount = best_buy.details.amount;
            let sell_amount = best_sell.details.amount;
            let matched_amount = std::cmp::min(buy_amount, sell_amount);

            let mut buy_order = self.buys.remove(0);
            let mut sell_order = self.sells.remove(0);

            // Handle partial fills — reinsert remainder
            if buy_amount > matched_amount {
                let mut residual = buy_order.clone();
                residual.details.amount = buy_amount - matched_amount;
                self.buys.insert(0, residual); // Already sorted position
            }
            if sell_amount > matched_amount {
                let mut residual = sell_order.clone();
                residual.details.amount = sell_amount - matched_amount;
                self.sells.insert(0, residual);
            }

            // Set matched orders to the actual matched amount
            buy_order.details.amount = matched_amount;
            sell_order.details.amount = matched_amount;

            return Some(MatchResult {
                buy: buy_order,
                sell: sell_order,
                execution_price,
                matched_amount,
            });
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_details(side: u8, price: u64, amount: u64) -> OrderDetails {
        OrderDetails {
            amount,
            side,
            price,
            nonce: [0u8; 32],
        }
    }

    #[test]
    fn test_exact_match() {
        let mut engine = MatchingEngine::new();
        engine.add_order([1u8; 32], Pubkey::default(), make_details(0, 100, 50));
        engine.add_order([2u8; 32], Pubkey::default(), make_details(1, 100, 50));

        let result = engine.match_orders().unwrap();
        assert_eq!(result.matched_amount, 50);
        assert!(engine.buys.is_empty());
        assert!(engine.sells.is_empty());
    }

    #[test]
    fn test_partial_fill_buy_larger() {
        let mut engine = MatchingEngine::new();
        engine.add_order([1u8; 32], Pubkey::default(), make_details(0, 100, 100));
        engine.add_order([2u8; 32], Pubkey::default(), make_details(1, 90, 30));

        let result = engine.match_orders().unwrap();
        assert_eq!(result.matched_amount, 30);
        assert_eq!(engine.buys.len(), 1);
        assert_eq!(engine.buys[0].details.amount, 70); // Remainder
        assert!(engine.sells.is_empty());
    }

    #[test]
    fn test_no_match_when_price_gap() {
        let mut engine = MatchingEngine::new();
        engine.add_order([1u8; 32], Pubkey::default(), make_details(0, 80, 50));
        engine.add_order([2u8; 32], Pubkey::default(), make_details(1, 100, 50));

        assert!(engine.match_orders().is_none());
    }
}
