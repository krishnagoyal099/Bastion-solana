#[macro_use]
extern crate honggfuzz;

use arbitrary::Arbitrary;
use bastion_pool::instructions::deposit_sol::*;
use bastion_pool::state::*;

#[derive(Arbitrary, Debug)]
struct FuzzInput {
    amount: u64,
}

fn main() {
    loop {
        fuzz!(|data: FuzzInput| {
            // Very basic stub to demonstrate how honggfuzz would feed arbitrary 
            // values into our instruction logic context.
            // In a real scenario, this involves spinning up a solana-program-test bank.
            let _amount = data.amount;
        });
    }
}
