use anchor_lang::prelude::*;

use crate::constants::*;
use crate::events::*;
use crate::state::*;

/// Creates a user profile to track challenge statistics
pub fn initialize_user(ctx: Context<InitializeUser>) -> Result<()> {
    let clock = Clock::get()?;

    ctx.accounts.user.set_inner(User {
        authority: ctx.accounts.authority.key(),
        total_challenges: 0,
        completed: 0,
        failed: 0,
        total_staked: 0,
        bump: ctx.bumps.user,
    });

    emit!(UserInitialized {
        authority: ctx.accounts.user.authority,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct InitializeUser<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + User::INIT_SPACE,
        seeds = [USER_SEED, authority.key().as_ref()],
        bump
    )]
    pub user: Account<'info, User>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}
