use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

use crate::constants::*;
use crate::errors::ErrorCode;
use crate::events::*;
use crate::state::*;

/// Creator cancels challenge before completion
pub fn cancel_challenge(ctx: Context<CancelChallenge>) -> Result<()> {
    let clock = Clock::get()?;
    let challenge = &mut ctx.accounts.challenge;
    let challenge_key = challenge.key();
    let creator = challenge.creator;
    let stake_amount = challenge.stake_amount;

    require!(challenge.creator == ctx.accounts.creator.key(), ErrorCode::UnauthorizedCreator);
    require!(
        challenge.status == ChallengeStatus::Created || challenge.status == ChallengeStatus::Active,
        ErrorCode::CannotCancelChallenge
    );

    let platform_fee = (challenge.stake_amount * PLATFORM_FEE_BPS) / BASIS_POINTS;
    let refund_amount: u64;
    let penalty: u64;

    // Calculate refund and penalty
    if challenge.status == ChallengeStatus::Created {
        // Full refund if challenge hasn't started
        refund_amount = challenge.stake_amount + platform_fee;
        penalty = 0;
    } else {
        // Active: penalty applies (2% of stake)
        penalty = (challenge.stake_amount * CANCEL_PENALTY_BPS) / BASIS_POINTS;
        refund_amount = challenge.stake_amount + platform_fee - penalty;
    }

    // Transfer refund to creator
    let escrow_seeds = &[
        ESCROW_SEED,
        challenge_key.as_ref(),
        &[challenge.escrow_bump],
    ];
    let signer_seeds = &[&escrow_seeds[..]];

    transfer(
        CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.escrow.to_account_info(),
                to: ctx.accounts.creator.to_account_info(),
            },
            signer_seeds,
        ),
        refund_amount,
    )?;

    // Transfer penalty to platform
    if penalty > 0 {
        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.escrow.to_account_info(),
                    to: ctx.accounts.platform.to_account_info(),
                },
                signer_seeds,
            ),
            penalty,
        )?;
    }

    challenge.status = ChallengeStatus::Cancelled;

    emit!(ChallengeCancelled {
        challenge: challenge_key,
        creator,
        stake_amount,
        refund_amount,
        penalty,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct CancelChallenge<'info> {
    #[account(mut)]
    pub challenge: Account<'info, Challenge>,
    #[account(
        mut,
        seeds = [ESCROW_SEED, challenge.key().as_ref()],
        bump = challenge.escrow_bump,
    )]
    /// CHECK: Escrow PDA
    pub escrow: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK: Platform wallet (receives penalty fees)
    pub platform: AccountInfo<'info>,
    #[account(mut)]
    pub creator: Signer<'info>,
    pub system_program: Program<'info, System>,
}
