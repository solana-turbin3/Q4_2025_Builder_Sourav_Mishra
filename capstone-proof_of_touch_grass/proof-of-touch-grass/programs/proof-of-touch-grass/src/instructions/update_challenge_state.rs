use anchor_lang::prelude::*;

use crate::constants::*;
use crate::errors::ErrorCode;
use crate::events::*;
use crate::state::*;

/// Updates challenge state based on time progression (admin-only)
/// - Created → Active (when start_time reached)
/// - Active → Failed (when end_time passed without enough evidence)
/// - Active → PendingVerification (when end_time passed with all evidence)
/// - PendingVerification → Completed (when verification_period_end passed - innocent until proven guilty)
pub fn update_challenge_state(ctx: Context<UpdateChallengeState>) -> Result<()> {
    let clock = Clock::get()?;
    let challenge = &mut ctx.accounts.challenge;
    let user = &mut ctx.accounts.user;
    let challenge_key = challenge.key();
    let creator = challenge.creator;

    // Admin-only check
    require!(
        ctx.accounts.admin.key() == ADMIN_PUBKEY,
        ErrorCode::UnauthorizedAdmin
    );

    let mut state_changed = false;
    let mut new_status = challenge.status.clone();
    let old_status = challenge.status.to_string();

    match challenge.status {
        ChallengeStatus::Created => {
            // Created → Active when start_time is reached
            if clock.unix_timestamp >= challenge.start_time {
                challenge.status = ChallengeStatus::Active;
                new_status = ChallengeStatus::Active;
                state_changed = true;
            }
        }
        ChallengeStatus::Active => {
            // Active → Failed/PendingVerification when end_time is passed
            if clock.unix_timestamp > challenge.end_time {
                if challenge.evidence_count < challenge.required_proofs {
                    // Not enough evidence submitted → Failed
                    challenge.status = ChallengeStatus::Failed;
                    challenge.finalized_at = clock.unix_timestamp;
                    new_status = ChallengeStatus::Failed;
                    user.failed += 1;
                    state_changed = true;
                } else {
                    // All evidence submitted → PendingVerification
                    challenge.status = ChallengeStatus::PendingVerification;
                    new_status = ChallengeStatus::PendingVerification;
                    state_changed = true;
                }
            }
        }
        ChallengeStatus::PendingVerification => {
            // PendingVerification → Completed when verification_period_end is passed
            if clock.unix_timestamp > challenge.verification_period_end {
                challenge.status = ChallengeStatus::Completed;
                challenge.finalized_at = clock.unix_timestamp;
                new_status = ChallengeStatus::Completed;
                user.completed += 1;
                state_changed = true;
            }
        }
        _ => {
            // No state transitions for Completed, Failed, Cancelled, or Disputed
        }
    }

    if state_changed {
        emit!(ChallengeStateUpdated {
            challenge: challenge_key,
            creator,
            old_status,
            new_status: new_status.to_string(),
            timestamp: clock.unix_timestamp,
        });
    }

    Ok(())
}

#[derive(Accounts)]
pub struct UpdateChallengeState<'info> {
    #[account(mut)]
    pub challenge: Account<'info, Challenge>,
    #[account(
        mut,
        seeds = [USER_SEED, challenge.creator.as_ref()],
        bump = user.bump,
    )]
    pub user: Account<'info, User>,
    pub admin: Signer<'info>,
}
