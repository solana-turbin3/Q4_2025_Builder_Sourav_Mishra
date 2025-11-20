use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Title exceeds maximum length")]
    TitleTooLong,
    #[msg("Description exceeds maximum length")]
    DescriptionTooLong,
    #[msg("Stake amount must be greater than 0")]
    InvalidStakeAmount,
    #[msg("End time must be after start time")]
    InvalidTimeRange,
    #[msg("Invalid number of verifiers (1-10 required)")]
    InvalidVerifierCount,
    #[msg("Required approvals cannot exceed verifier count")]
    InvalidApprovalCount,
    #[msg("Required proofs must be greater than 0")]
    InvalidProofCount,
    #[msg("IPFS hash exceeds maximum length")]
    IpfsHashTooLong,
    #[msg("Metadata exceeds maximum length")]
    MetadataTooLong,
    #[msg("Only challenge creator can submit evidence")]
    UnauthorizedSubmitter,
    #[msg("Challenge is not in the correct status for this operation")]
    InvalidChallengeStatus,
    #[msg("Challenge has expired")]
    ChallengeExpired,
    #[msg("All required evidence has been submitted")]
    AllEvidenceSubmitted,
    #[msg("Verification period has expired")]
    VerificationPeriodExpired,
    #[msg("Not authorized as a verifier for this challenge")]
    UnauthorizedVerifier,
    #[msg("Verification period has not expired yet")]
    VerificationPeriodNotExpired,
    #[msg("Only challenge creator can cancel")]
    UnauthorizedCreator,
    #[msg("Cannot cancel challenge in current status")]
    CannotCancelChallenge,
    #[msg("Dispute reason exceeds maximum length")]
    DisputeReasonTooLong,
    #[msg("Dispute window has expired")]
    DisputeWindowExpired,
    #[msg("Only creator or voting verifiers can dispute")]
    UnauthorizedDisputer,
    #[msg("Verifier did not vote to reject")]
    VerifierDidNotReject,
    #[msg("Rewards already claimed")]
    AlreadyClaimed,
    #[msg("Dispute window has not expired yet")]
    DisputeWindowNotExpired,
    #[msg("Only admin can update challenge state")]
    UnauthorizedAdmin,
    #[msg("Verifier has already voted")]
    AlreadyVoted,
}
