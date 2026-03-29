// withdraw_fees: authority drains accumulated SOL fees held by the curve PDA.

use crate::{errors::SwifeyError, state::{BondingCurve, Config}};
use crate::utils::sol_transfer_with_signer;
use anchor_lang::{prelude::*, system_program};
use anchor_spl::token::Mint;

#[derive(Accounts)]
pub struct WithdrawFees<'info> {
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

    /// CHECK: Verified against global_config.fee_recipient
    #[account(
        mut,
        constraint = global_config.fee_recipient == fee_recipient.key() @ SwifeyError::IncorrectFeeRecipient
    )]
    pub fee_recipient: AccountInfo<'info>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

impl<'info> WithdrawFees<'info> {
    pub fn process(&mut self, bump_bonding_curve: u8) -> Result<()> {
        let curve_pda = &self.bonding_curve.to_account_info();

        // Rent-exempt minimum must remain in the PDA to keep it alive
        let rent = Rent::get()?;
        let min_lamports = rent.minimum_balance(curve_pda.data_len());
        let available = curve_pda
            .lamports()
            .checked_sub(min_lamports)
            .ok_or(SwifeyError::InsufficientSolBalance)?;

        require!(available > 0, SwifeyError::InsufficientSolBalance);

        let token_key = self.token_mint.key();
        let seeds = BondingCurve::get_signer(&token_key, &bump_bonding_curve);
        let signer_seeds: &[&[&[u8]]] = &[&seeds];

        sol_transfer_with_signer(
            curve_pda,
            &self.fee_recipient,
            &self.system_program.to_account_info(),
            signer_seeds,
            available,
        )?;

        Ok(())
    }
}