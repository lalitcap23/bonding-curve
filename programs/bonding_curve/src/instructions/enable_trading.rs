// enable_trading: authority flips the trading gate on a bonding curve.
// Swaps check `is_trading_enabled` before executing.

use crate::{errors::SwifeyError, state::{BondingCurve, Config}};
use anchor_lang::{prelude::*, system_program};
use anchor_spl::token::Mint;

#[derive(Accounts)]
pub struct EnableTrading<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump,
        constraint = global_config.authority == authority.key() @ SwifeyError::UnauthorizedAddress,
    )]
    pub global_config: Account<'info, Config>,

    pub token_mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [BondingCurve::SEED_PREFIX.as_bytes(), token_mint.key().as_ref()],
        bump,
    )]
    pub bonding_curve: Account<'info, BondingCurve>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

impl<'info> EnableTrading<'info> {
    pub fn process(&mut self, enable: bool) -> Result<()> {
        // Prevent toggling after the curve has completed/migrated
        require!(
            !self.bonding_curve.is_completed,
            SwifeyError::CurveLimitReached
        );
        require!(
            !self.bonding_curve.is_migrated,
            SwifeyError::AlreadyMigrated
        );

        self.bonding_curve.is_trading_enabled = enable;
        Ok(())
    }
}