pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("8QZyPMwm2tMo35mDVXrWYTEYrXeRn7VEYKycnTiiU6P3");

#[program]
pub mod bonding_curve {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        initi_config::handler(ctx)
    }
}
