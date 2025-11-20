use anchor_lang::prelude::*;
use crate::constants::*;

/// User profile and statistics
#[account]
#[derive(InitSpace)]
pub struct User {
    pub authority: Pubkey,
    pub total_challenges: u32,
    pub completed: u32,
    pub failed: u32,
    pub total_staked: u64,
    pub bump: u8,
}

/// Challenge with stake parameters
#[account]
#[derive(InitSpace)]
pub struct Challenge {
    pub creator: Pubkey,
    #[max_len(MAX_TITLE_LEN)]
    pub title: String,
    #[max_len(MAX_DESCRIPTION_LEN)]
    pub description: String,
    pub stake_amount: u64,
    pub start_time: i64,
    pub end_time: i64,
    pub verification_period_end: i64,
    pub required_proofs: u8,
    pub required_approvals: u8,
    #[max_len(MAX_VERIFIERS)]
    pub verifiers: Vec<Pubkey>,
    pub status: ChallengeStatus,
    pub evidence_count: u8,
    pub approval_count: u8,
    pub rejection_count: u8,
    pub finalized_at: i64,
    pub claimed: bool,
    pub bump: u8,
    pub escrow_bump: u8,
}

/// Submitted evidence with metadata
#[account]
#[derive(InitSpace)]
pub struct Evidence {
    pub challenge: Pubkey,
    #[max_len(MAX_IPFS_HASH_LEN)]
    pub ipfs_hash: String,
    #[max_len(MAX_METADATA_LEN)]
    pub metadata: String,
    pub timestamp: i64,
    pub evidence_index: u8,
    pub bump: u8,
}

/// Verifier's vote on evidence
#[account]
#[derive(InitSpace)]
pub struct Verification {
    pub challenge: Pubkey,
    pub verifier: Pubkey,
    pub vote: Vote,
    pub timestamp: i64,
    pub claimed: bool,
    pub bump: u8,
}

/// Dispute filed against verification
#[account]
#[derive(InitSpace)]
pub struct Dispute {
    pub challenge: Pubkey,
    pub disputer: Pubkey,
    #[max_len(MAX_DISPUTE_REASON_LEN)]
    pub reason: String,
    pub timestamp: i64,
    pub bump: u8,
}

/// Challenge lifecycle state
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum ChallengeStatus {
    Created,
    Active,
    PendingVerification,
    Completed,
    Failed,
    Cancelled,
    Disputed,
}

impl ChallengeStatus {
    pub fn to_string(&self) -> String {
        match self {
            ChallengeStatus::Created => "Created".to_string(),
            ChallengeStatus::Active => "Active".to_string(),
            ChallengeStatus::PendingVerification => "PendingVerification".to_string(),
            ChallengeStatus::Completed => "Completed".to_string(),
            ChallengeStatus::Failed => "Failed".to_string(),
            ChallengeStatus::Cancelled => "Cancelled".to_string(),
            ChallengeStatus::Disputed => "Disputed".to_string(),
        }
    }
}

/// Verifier decision type
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum Vote {
    Approve,
    Reject,
}

impl Vote {
    pub fn to_string(&self) -> String {
        match self {
            Vote::Approve => "Approve".to_string(),
            Vote::Reject => "Reject".to_string(),
        }
    }
}
