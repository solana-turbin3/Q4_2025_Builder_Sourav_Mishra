use anchor_lang::prelude::*;

use crate::constants::*;
use crate::errors::ErrorCode;
use crate::events::*;
use crate::state::*;

/// Verifiers vote to approve or reject evidence
pub fn verify_evidence(
    ctx: Context<VerifyEvidence>,
    vote: Vote,
) -> Result<()> {
    let clock = Clock::get()?;
    let challenge = &mut ctx.accounts.challenge;
    let user = &mut ctx.accounts.user;
    let challenge_key = challenge.key();
    let verifier_key = ctx.accounts.verifier.key();

    require!(challenge.status == ChallengeStatus::PendingVerification, ErrorCode::InvalidChallengeStatus);
    require!(clock.unix_timestamp < challenge.verification_period_end, ErrorCode::VerificationPeriodExpired);
    require!(
        challenge.verifiers.contains(&verifier_key),
        ErrorCode::UnauthorizedVerifier
    );

    ctx.accounts.verification.set_inner(Verification {
        challenge: challenge_key,
        verifier: verifier_key,
        vote: vote.clone(),
        timestamp: clock.unix_timestamp,
        claimed: false,
        bump: ctx.bumps.verification,
    });

    // Update vote counts
    match vote {
        Vote::Approve => {
            challenge.approval_count += 1;
        }
        Vote::Reject => {
            challenge.rejection_count += 1;
        }
    }

    let total_verifiers = challenge.verifiers.len() as u8;
    let required_approvals = challenge.required_approvals;
    let max_possible_rejections = total_verifiers - required_approvals;

    // Check if approval threshold met → Completed
    if challenge.approval_count >= required_approvals {
        challenge.status = ChallengeStatus::Completed;
        challenge.finalized_at = clock.unix_timestamp;
        user.completed += 1;

        emit!(ChallengeFinalized {
            challenge: challenge_key,
            creator: challenge.creator,
            status: "Completed".to_string(),
            approval_count: challenge.approval_count,
            rejection_count: challenge.rejection_count,
            required_approvals,
            timestamp: clock.unix_timestamp,
        });
    }
    // Check if rejection threshold met → Failed
    else if challenge.rejection_count > max_possible_rejections {
        challenge.status = ChallengeStatus::Failed;
        challenge.finalized_at = clock.unix_timestamp;
        user.failed += 1;

        emit!(ChallengeFinalized {
            challenge: challenge_key,
            creator: challenge.creator,
            status: "Failed".to_string(),
            approval_count: challenge.approval_count,
            rejection_count: challenge.rejection_count,
            required_approvals,
            timestamp: clock.unix_timestamp,
        });
    }

    emit!(VoteCast {
        challenge: challenge_key,
        verifier: verifier_key,
        vote: ctx.accounts.verification.vote.to_string(),
        approval_count: challenge.approval_count,
        rejection_count: challenge.rejection_count,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct VerifyEvidence<'info> {
    #[account(
        init,
        payer = verifier,
        space = 8 + Verification::INIT_SPACE,
        seeds = [VERIFICATION_SEED, challenge.key().as_ref(), verifier.key().as_ref()],
        bump
    )]
    pub verification: Account<'info, Verification>,
    #[account(mut)]
    pub challenge: Account<'info, Challenge>,
    #[account(
        mut,
        seeds = [USER_SEED, challenge.creator.as_ref()],
        bump = user.bump,
    )]
    pub user: Account<'info, User>,
    #[account(mut)]
    pub verifier: Signer<'info>,
    pub system_program: Program<'info, System>,
}
