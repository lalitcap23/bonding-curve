// set_params: authority-gated update of protocol fee rates and curve limit.

use crate::{errors::SwifeyError, state::Config};
use anchor_lang::{prelude::*, system_program};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct SetParamsArgs {
    pub buy_fee_percentage: f64,
    pub sell_fee_percentage: f64,
    pub migration_fee_percentage: f64,
    pub curve_limit: u64,
    pub fee_recipient: Pubkey,
}

#[derive(Accounts)]
pub struct SetParams<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump,
        constraint = global_config.authority == authority.key() @ SwifeyError::UnauthorizedAddress,
    )]
    pub global_config: Account<'info, Config>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

impl<'info> SetParams<'info> {
    pub fn process(&mut self, args: SetParamsArgs) -> Result<()> {
        // Fee percentages must be within 0-100
        require!(
            args.buy_fee_percentage >= 0.0 && args.buy_fee_percentage <= 100.0,
            SwifeyError::IncorrectValueRange
        );
        require!(
            args.sell_fee_percentage >= 0.0 && args.sell_fee_percentage <= 100.0,
            SwifeyError::IncorrectValueRange
        );
        require!(
            args.migration_fee_percentage >= 0.0 && args.migration_fee_percentage <= 100.0,
            SwifeyError::IncorrectValueRange
        );

        // curve_limit must be a positive value
        require!(args.curve_limit > 0, SwifeyError::IncorrectValueRange);

        // fee_recipient must not be the default/zero pubkey
        require!(
            !args.fee_recipient.eq(&Pubkey::default()),
            SwifeyError::UnauthorizedAddress
        );

        let cfg = &mut self.global_config;
        cfg.buy_fee_percentage = args.buy_fee_percentage;
        cfg.sell_fee_percentage = args.sell_fee_percentage;
        cfg.migration_fee_percentage = args.migration_fee_percentage;
        cfg.curve_limit = args.curve_limit;
        cfg.fee_recipient = args.fee_recipient;

        Ok(())
    }
}
