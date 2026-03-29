use anchor_lang::prelude::*;

#[event]
pub struct MigrationCompleted {
    pub token_mint: Pubkey,
    pub sol_amount: u64,
    pub token_amount: u64,
    pub migration_fee: u64,
    pub raydium_pool: Pubkey,
}

#[event]
pub struct TokenSold {
    pub token_mint: Pubkey,
    pub sol_amount: u64,
    pub token_amount: u64,
    pub fee_amount: u64,
    pub price: u64
}

#[event]
pub struct TokenPurchased {
    pub token_mint: Pubkey,
    pub buyer: Pubkey,
    pub sol_amount: u64,
    pub token_amount: u64,
    pub fee_amount: u64,
    pub price: u64
}

#[event]
pub struct CurveCompleted {
    pub token_mint: Pubkey,
    pub final_sol_reserve: u64,
    pub final_token_reserve: u64,
}
