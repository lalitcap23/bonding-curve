use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint};
use crate::states::{BondingCurve, Config};
use crate::errors::SwifeyError;
use crate::utils::{sol_transfer_with_signer, token_transfer_with_signer, MigrationCompleted};
use raydium_amm_v3::{
    self,
    states::{AmmConfig, POOL_SEED, POOL_TICK_ARRAY_BITMAP_SEED, POOL_VAULT_SEED},
    program::AmmV3,
    libraries::tick_math,
};

#[derive(Accounts)]
pub struct Migrate<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump,
    )]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        seeds = [BondingCurve::SEED_PREFIX.as_bytes(), token_mint.key().as_ref()],
        bump,
        constraint = bonding_curve.is_completed @ SwifeyError::CurveNotCompleted,
        constraint = !bonding_curve.is_migrated @ SwifeyError::AlreadyMigrated,
    )]
    pub bonding_curve: Account<'info, BondingCurve>,

    #[account(mut)]
    pub token_mint: Account<'info, Mint>,

    /// CHECK: WSOL mint
    pub wsol_mint: Account<'info, Mint>,

    #[account(
        mut,
        constraint = curve_token_account.owner == bonding_curve.key(),
        constraint = curve_token_account.amount > 0 @ SwifeyError::InsufficientTokenBalance,
    )]
    pub curve_token_account: Account<'info, TokenAccount>,

    /// CHECK: Native SOL account owned by bonding curve
    #[account(
        mut,
        constraint = *curve_sol_account.owner == bonding_curve.key(),
        constraint = curve_sol_account.lamports() > 0 @ SwifeyError::InsufficientSolBalance,
    )]
    pub curve_sol_account: AccountInfo<'info>,

    /// CHECK: Pool state account to be created
    #[account(
        mut,
        seeds = [
            POOL_SEED.as_bytes(),
            amm_config.key().as_ref(),
            wsol_mint.key().as_ref(),
            token_mint.key().as_ref(),
        ],
        seeds::program = raydium_program,
        bump,
    )]
    pub pool_state: UncheckedAccount<'info>,

    /// CHECK: Pool observation state account
    #[account(mut)]
    pub observation_state: UncheckedAccount<'info>,

    /// CHECK: Token vault for WSOL
    #[account(
        mut,
        seeds = [
            POOL_VAULT_SEED.as_bytes(),
            pool_state.key().as_ref(),
            wsol_mint.key().as_ref(),
        ],
        seeds::program = raydium_program,
        bump,
    )]
    pub token_vault_0: UncheckedAccount<'info>,

    /// CHECK: Token vault for project token
    #[account(
        mut,
        seeds = [
            POOL_VAULT_SEED.as_bytes(),
            pool_state.key().as_ref(),
            token_mint.key().as_ref(),
        ],
        seeds::program = raydium_program,
        bump,
    )]
    pub token_vault_1: UncheckedAccount<'info>,

    /// CHECK: Initialize an account to store if a tick array is initialized
    #[account(
        mut,
        seeds = [
            POOL_TICK_ARRAY_BITMAP_SEED.as_bytes(),
            pool_state.key().as_ref(),
        ],
        seeds::program = raydium_program,
        bump,
    )]
    pub tick_array_bitmap: UncheckedAccount<'info>,

    /// CHECK: Fee recipient account
    #[account(mut)]
    pub fee_recipient: AccountInfo<'info>,

    /// CHECK: Raydium program state account
    pub amm_config: Box<Account<'info, AmmConfig>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub raydium_program: Program<'info, AmmV3>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> Migrate<'info> {
    pub fn process(ctx: Context<Migrate>) -> Result<()> {
        let bonding_curve = &mut ctx.accounts.bonding_curve;
        let config = &ctx.accounts.config;
        
        // Verify authority
        require!(
            config.authority == ctx.accounts.authority.key(),
            SwifeyError::UnauthorizedAddress
        );

        // Calculate amounts and fees
        let sol_balance = ctx.accounts.curve_sol_account.lamports();
        let token_balance = ctx.accounts.curve_token_account.amount;
        
        // Calculate migration fee
        let migration_fee = sol_balance
            .checked_mul(config.migration_fee_percentage as u64)
            .ok_or(SwifeyError::MathOverflow)?
            .checked_div(100)
            .ok_or(SwifeyError::MathOverflow)?;
        
        let remaining_sol = sol_balance
            .checked_sub(migration_fee)
            .ok_or(SwifeyError::InsufficientSolBalance)?;

        // Create Raydium pool
        let init_sqrt_price = tick_math::get_sqrt_price_at_tick(0)?; // Start at tick 0 for 1:1 price
        let open_time = Clock::get()?.unix_timestamp as u64;
        
        let create_pool_accounts = raydium_amm_v3::cpi::accounts::CreatePool {
            pool_creator: ctx.accounts.authority.to_account_info(),
            amm_config: ctx.accounts.amm_config.to_account_info(),
            pool_state: ctx.accounts.pool_state.to_account_info(),
            token_mint_0: ctx.accounts.wsol_mint.to_account_info(),
            token_mint_1: ctx.accounts.token_mint.to_account_info(),
            token_vault_0: ctx.accounts.token_vault_0.to_account_info(),
            token_vault_1: ctx.accounts.token_vault_1.to_account_info(),
            observation_state: ctx.accounts.observation_state.to_account_info(),
            tick_array_bitmap: ctx.accounts.tick_array_bitmap.to_account_info(),
            token_program_0: ctx.accounts.token_program.to_account_info(),
            token_program_1: ctx.accounts.token_program.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            rent: ctx.accounts.rent.to_account_info(),
        };

        let create_pool_ctx = CpiContext::new(
            ctx.accounts.raydium_program.to_account_info(),
            create_pool_accounts,
        );

        raydium_amm_v3::cpi::create_pool(
            create_pool_ctx,
            init_sqrt_price,
            open_time,
        )?;

        // Get signer seeds for PDA operations
        let bump = ctx.bumps.bonding_curve;
        let token_key = ctx.accounts.token_mint.key();
        let seeds = BondingCurve::get_signer(
            &token_key,
            &bump
        );
        let signer_seeds = &[&seeds[..]];
    
        // Transfer migration fee to fee recipient
        sol_transfer_with_signer(
            &ctx.accounts.curve_sol_account.to_account_info(),
            &ctx.accounts.fee_recipient,
            &ctx.accounts.system_program,
            signer_seeds,
            migration_fee,
        )?;
    
        // Transfer remaining SOL to Raydium pool
        sol_transfer_with_signer(
            &ctx.accounts.curve_sol_account.to_account_info(),
            &ctx.accounts.token_vault_0.to_account_info(), // Transfer to WSOL vault
            &ctx.accounts.system_program,
            signer_seeds,
            remaining_sol,
        )?;
    
        // Transfer tokens to Raydium pool
        token_transfer_with_signer(
            &ctx.accounts.curve_token_account.to_account_info(),
            &bonding_curve.to_account_info(),
            &ctx.accounts.token_vault_1.to_account_info(), // Transfer to token vault
            &ctx.accounts.token_program.to_account_info(),
            signer_seeds,
            token_balance,
        )?;
    
        // Mark as migrated
        bonding_curve.is_migrated = true;
    
        emit!(MigrationCompleted{
            token_mint: ctx.accounts.token_mint.key(),
            sol_amount: remaining_sol,
            token_amount: token_balance,
            migration_fee: migration_fee,
            raydium_pool: ctx.accounts.pool_state.key(),
        });
        
        Ok(())
    }
}