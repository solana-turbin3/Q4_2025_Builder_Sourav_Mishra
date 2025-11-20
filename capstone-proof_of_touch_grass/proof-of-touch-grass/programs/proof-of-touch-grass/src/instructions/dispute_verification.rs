use anchor_lang::prelude::*;

use crate::constants::*;
use crate::errors::ErrorCode;
use crate::events::*;
use crate::state::*;

/// Disputes verification result within dispute window
pub fn dispute_verification(
    ctx: Context<DisputeVerification>,
    reason: String,
) -> Result<()> {
    require!(reason.len() <= MAX_DISPUTE_REASON_LEN, ErrorCode::DisputeReasonTooLong);

    let clock = Clock::get()?;
    let challenge = &mut ctx.accounts.challenge;
    let challenge_key = challenge.key();
    let disputer_key = ctx.accounts.disputer.key();

    // Validations
    require!(
        challenge.status == ChallengeStatus::Completed || challenge.status == ChallengeStatus::Failed,
        ErrorCode::InvalidChallengeStatus
    );
    require!(
        clock.unix_timestamp <= challenge.finalized_at + DISPUTE_WINDOW,
        ErrorCode::DisputeWindowExpired
    );

    // Check if disputer is creator or a verifier who voted
    let is_creator = challenge.creator == disputer_key;
    let is_verifier = challenge.verifiers.contains(&disputer_key);
    require!(is_creator || is_verifier, ErrorCode::UnauthorizedDisputer);

    let previous_status = challenge.status.to_string();

    ctx.accounts.dispute.set_inner(Dispute {
        challenge: challenge_key,
        disputer: disputer_key,
        reason: reason.clone(),
        timestamp: clock.unix_timestamp,
        bump: ctx.bumps.dispute,
    });

    // Lock escrow and update status
    challenge.status = ChallengeStatus::Disputed;

    emit!(DisputeFiled {
        challenge: challenge_key,
        disputer: disputer_key,
        previous_status,
        reason,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct DisputeVerification<'info> {
    #[account(
        init,
        payer = disputer,
        space = 8 + Dispute::INIT_SPACE,
        seeds = [DISPUTE_SEED, challenge.key().as_ref()],
        bump
    )]
    pub dispute: Account<'info, Dispute>,
    #[account(mut)]
    pub challenge: Account<'info, Challenge>,
    #[account(mut)]
    pub disputer: Signer<'info>,
    pub system_program: Program<'info, System>,
}
