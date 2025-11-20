use anchor_lang::prelude::*;

#[event]
pub struct UserInitialized {
    pub authority: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct ChallengeCreated {
    pub challenge: Pubkey,
    pub creator: Pubkey,
    pub title: String,
    pub stake_amount: u64,
    pub platform_fee: u64,
    pub start_time: i64,
    pub end_time: i64,
    pub required_proofs: u8,
    pub required_approvals: u8,
    pub verifier_count: u8,
    pub timestamp: i64,
}

#[event]
pub struct EvidenceSubmitted {
    pub challenge: Pubkey,
    pub evidence: Pubkey,
    pub submitter: Pubkey,
    pub ipfs_hash: String,
    pub evidence_index: u8,
    pub total_evidence: u8,
    pub required_proofs: u8,
    pub timestamp: i64,
}

#[event]
pub struct VoteCast {
    pub challenge: Pubkey,
    pub verifier: Pubkey,
    pub vote: String, // "Approve" or "Reject"
    pub approval_count: u8,
    pub rejection_count: u8,
    pub timestamp: i64,
}

#[event]
pub struct ChallengeFinalized {
    pub challenge: Pubkey,
    pub creator: Pubkey,
    pub status: String, // "Completed" or "Failed"
    pub approval_count: u8,
    pub rejection_count: u8,
    pub required_approvals: u8,
    pub timestamp: i64,
}

#[event]
pub struct ChallengeCancelled {
    pub challenge: Pubkey,
    pub creator: Pubkey,
    pub stake_amount: u64,
    pub refund_amount: u64,
    pub penalty: u64,
    pub timestamp: i64,
}

#[event]
pub struct DisputeFiled {
    pub challenge: Pubkey,
    pub disputer: Pubkey,
    pub previous_status: String,
    pub reason: String,
    pub timestamp: i64,
}

#[event]
pub struct RewardsClaimed {
    pub challenge: Pubkey,
    pub verifier: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct SuccessRewardClaimed {
    pub challenge: Pubkey,
    pub creator: Pubkey,
    pub stake_amount: u64,
    pub bonus: u64,
    pub total_amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct ChallengeStateUpdated {
    pub challenge: Pubkey,
    pub creator: Pubkey,
    pub old_status: String,
    pub new_status: String,
    pub timestamp: i64,
}

#[event]
pub struct FundsClaimed {
    pub challenge: Pubkey,
    pub claimer: Pubkey,
    pub amount: u64,
    pub platform_fee: u64,
    pub challenge_status: String,
    pub timestamp: i64,
}
