pub mod constants;
pub mod errors;
pub mod instructions;
pub mod state;
pub mod utils;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("8QZyPMwm2tMo35mDVXrWYTEYrXeRn7VEYKycnTiiU6P3");

#[program]
pub mod bonding_curve {
    use super::*;

    //  Admin / Configuration 

    /// Initialize or update the global protocol config (admin-only).
    pub fn configure(ctx: Context<Configure>, new_config: state::ConfigSettings) -> Result<()> {
        ctx.accounts.process(new_config)
    }

    /// Update individual protocol params (fees, curve_limit, fee_recipient).
    pub fn set_params(ctx: Context<SetParams>, args: instructions::set_params::SetParamsArgs) -> Result<()> {
        ctx.accounts.process(args)
    }

    /// Open or close swaps for a specific bonding curve instance.
    pub fn enable_trading(ctx: Context<EnableTrading>, enable: bool) -> Result<()> {
        ctx.accounts.process(enable)
    }

    /// Drain surplus SOL sitting in the curve PDA to the fee recipient.
    pub fn withdraw_fees(ctx: Context<WithdrawFees>, bump_bonding_curve: u8) -> Result<()> {
        ctx.accounts.process(bump_bonding_curve)
    }

    //Token / Curve Lifecycle 

    /// Create a new token mint + bonding curve PDA and seed it with the full supply.
    pub fn launch(
        ctx: Context<Launch>,
        name: String,
        symbol: String,
        uri: String,
        bump_config: u8,
    ) -> Result<()> {
        ctx.accounts.process(name, symbol, uri, bump_config)
    }

    /// Buy (direction = 0) or sell (direction = 1) on the bonding curve.
    pub fn swap(
        ctx: Context<Swap>,
        amount: u64,
        direction: u8,
        min_out: u64,
        bump_bonding_curve: u8,
    ) -> Result<()> {
        ctx.accounts.process(amount, direction, min_out, bump_bonding_curve)
    }

    /// Migrate a completed curve's liquidity to a Raydium CLMM pool.
    #[cfg(feature = "migration")]
    pub fn migrate(ctx: Context<Migrate>) -> Result<()> {
        Migrate::process(ctx)
    }
}
