use anchor_lang::prelude::*;

// Admin pubkey (authorized to update challenge states)
pub const ADMIN_PUBKEY: Pubkey = pubkey!("6wZQRRCWHeJriMqBpmpZF2PrJ3oVyTzEZMD5F5n388HU");

// Percentages in basis points (100 bp = 1%)
pub const PLATFORM_FEE_BPS: u64 = 50; // 0.5% of stake
pub const CANCEL_PENALTY_BPS: u64 = 200; // 2% of stake
pub const CREATOR_BONUS_BPS: u64 = 25; // 0.25% of stake (half of platform fee)
pub const SLASH_PENALTY_BPS: u64 = 2_500; // 25% of stake (slashed on failure)
pub const BASIS_POINTS: u64 = 10_000; // 100% = 10,000 basis points

// Time constants
pub const DISPUTE_WINDOW: i64 = 172800; // 48 hours

// Size limits
pub const MAX_VERIFIERS: usize = 5;
pub const MAX_TITLE_LEN: usize = 100;
pub const MAX_DESCRIPTION_LEN: usize = 500;
pub const MAX_IPFS_HASH_LEN: usize = 64;
pub const MAX_METADATA_LEN: usize = 200;
pub const MAX_DISPUTE_REASON_LEN: usize = 500;

// PDA Seeds
pub const USER_SEED: &[u8] = b"user";
pub const CHALLENGE_SEED: &[u8] = b"challenge";
pub const ESCROW_SEED: &[u8] = b"escrow";
pub const EVIDENCE_SEED: &[u8] = b"evidence";
pub const VERIFICATION_SEED: &[u8] = b"verification";
pub const DISPUTE_SEED: &[u8] = b"dispute";
