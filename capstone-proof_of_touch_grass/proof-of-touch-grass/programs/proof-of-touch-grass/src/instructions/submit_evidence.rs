use anchor_lang::prelude::*;

use crate::constants::*;
use crate::errors::ErrorCode;
use crate::events::*;
use crate::state::*;

/// Submits evidence for a challenge
pub fn submit_evidence(
    ctx: Context<SubmitEvidence>,
    ipfs_hash: String,
    metadata: String,
) -> Result<()> {
    require!(ipfs_hash.len() <= MAX_IPFS_HASH_LEN, ErrorCode::IpfsHashTooLong);
    require!(metadata.len() <= MAX_METADATA_LEN, ErrorCode::MetadataTooLong);

    let clock = Clock::get()?;
    let challenge = &mut ctx.accounts.challenge;
    let challenge_key = challenge.key();

    require!(challenge.creator == ctx.accounts.submitter.key(), ErrorCode::UnauthorizedSubmitter);
    require!(
        challenge.status == ChallengeStatus::Active,
        ErrorCode::InvalidChallengeStatus
    );
    require!(clock.unix_timestamp < challenge.end_time, ErrorCode::ChallengeExpired);
    require!(challenge.evidence_count < challenge.required_proofs, ErrorCode::AllEvidenceSubmitted);

    let evidence_index = challenge.evidence_count;

    ctx.accounts.evidence.set_inner(Evidence {
        challenge: challenge_key,
        ipfs_hash: ipfs_hash.clone(),
        metadata,
        timestamp: clock.unix_timestamp,
        evidence_index,
        bump: ctx.bumps.evidence,
    });

    challenge.evidence_count += 1;

    // If all evidence submitted, transition to PendingVerification status
    if challenge.evidence_count == challenge.required_proofs {
        challenge.status = ChallengeStatus::PendingVerification;
    }

    emit!(EvidenceSubmitted {
        challenge: challenge_key,
        evidence: ctx.accounts.evidence.key(),
        submitter: ctx.accounts.submitter.key(),
        ipfs_hash,
        evidence_index,
        total_evidence: challenge.evidence_count,
        required_proofs: challenge.required_proofs,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct SubmitEvidence<'info> {
    #[account(
        init,
        payer = submitter,
        space = 8 + Evidence::INIT_SPACE,
        seeds = [EVIDENCE_SEED, challenge.key().as_ref(), &challenge.evidence_count.to_le_bytes()],
        bump
    )]
    pub evidence: Account<'info, Evidence>,
    #[account(mut)]
    pub challenge: Account<'info, Challenge>,
    #[account(mut)]
    pub submitter: Signer<'info>,
    pub system_program: Program<'info, System>,
}
