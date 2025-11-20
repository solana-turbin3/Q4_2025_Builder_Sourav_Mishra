use anchor_lang::prelude::*;

declare_id!("5RR71WDjqBEwcuNT5AALw5y3nXFKL1sDJT3SHpkQmjm2");

pub mod constants;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod state;

use instructions::*;

#[program]
pub mod proof_of_touch_grass {
    use super::*;

    /// Creates a user profile to track challenge statistics
    pub fn initialize_user(ctx: Context<InitializeUser>) -> Result<()> {
        instructions::initialize_user::initialize_user(ctx)
    }

    /// Creates a challenge with stake and parameters
    pub fn create_challenge(
        ctx: Context<CreateChallenge>,
        title: String,
        description: String,
        stake_amount: u64,
        start_time: i64,
        end_time: i64,
        verification_period: i64,
        required_proofs: u8,
        required_approvals: u8,
        verifiers: Vec<Pubkey>,
    ) -> Result<()> {
        instructions::create_challenge::create_challenge(
            ctx,
            title,
            description,
            stake_amount,
            start_time,
            end_time,
            verification_period,
            required_proofs,
            required_approvals,
            verifiers,
        )
    }

    /// Admin-only: Updates challenge state based on time progression
    pub fn update_challenge_state(ctx: Context<UpdateChallengeState>) -> Result<()> {
        instructions::update_challenge_state::update_challenge_state(ctx)
    }

    /// Submits evidence for a challenge
    pub fn submit_evidence(
        ctx: Context<SubmitEvidence>,
        ipfs_hash: String,
        metadata: String,
    ) -> Result<()> {
        instructions::submit_evidence::submit_evidence(ctx, ipfs_hash, metadata)
    }

    /// Verifiers vote to approve or reject evidence
    pub fn verify_evidence(
        ctx: Context<VerifyEvidence>,
        vote: state::Vote,
    ) -> Result<()> {
        instructions::verify_evidence::verify_evidence(ctx, vote)
    }

    /// Creator cancels challenge before completion
    pub fn cancel_challenge(ctx: Context<CancelChallenge>) -> Result<()> {
        instructions::cancel_challenge::cancel_challenge(ctx)
    }

    /// Disputes verification result within dispute window
    pub fn dispute_verification(
        ctx: Context<DisputeVerification>,
        reason: String,
    ) -> Result<()> {
        instructions::dispute_verification::dispute_verification(ctx, reason)
    }

    /// Claim funds after challenge finalization (creator or verifiers)
    pub fn claim_funds(ctx: Context<ClaimFunds>) -> Result<()> {
        instructions::claim_funds::claim_funds(ctx)
    }
}
