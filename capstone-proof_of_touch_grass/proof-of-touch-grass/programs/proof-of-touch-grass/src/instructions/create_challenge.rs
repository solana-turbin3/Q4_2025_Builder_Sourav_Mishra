use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

use crate::constants::*;
use crate::errors::ErrorCode;
use crate::events::*;
use crate::state::*;

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
    require!(title.len() <= MAX_TITLE_LEN, ErrorCode::TitleTooLong);
    require!(description.len() <= MAX_DESCRIPTION_LEN, ErrorCode::DescriptionTooLong);
    require!(stake_amount > 0, ErrorCode::InvalidStakeAmount);
    require!(end_time > start_time, ErrorCode::InvalidTimeRange);
    require!(verifiers.len() > 0 && verifiers.len() <= MAX_VERIFIERS, ErrorCode::InvalidVerifierCount);
    require!(required_approvals as usize <= verifiers.len(), ErrorCode::InvalidApprovalCount);
    require!(required_proofs > 0, ErrorCode::InvalidProofCount);

    let user = &mut ctx.accounts.user;
    let clock = Clock::get()?;

    ctx.accounts.challenge.set_inner(Challenge {
        creator: ctx.accounts.creator.key(),
        title: title.clone(),
        description,
        stake_amount,
        start_time,
        end_time,
        verification_period_end: end_time + verification_period,
        required_proofs,
        required_approvals,
        verifiers: verifiers.clone(),
        status: ChallengeStatus::Created,
        evidence_count: 0,
        approval_count: 0,
        rejection_count: 0,
        finalized_at: 0,
        claimed: false,
        bump: ctx.bumps.challenge,
        escrow_bump: ctx.bumps.escrow,
    });

    // Calculate platform fee
    let platform_fee = (stake_amount * PLATFORM_FEE_BPS) / BASIS_POINTS;

    // Transfer stake + platform fee to escrow
    let total_amount = stake_amount + platform_fee;
    transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.creator.to_account_info(),
                to: ctx.accounts.escrow.to_account_info(),
            },
        ),
        total_amount,
    )?;

    // Update user stats
    user.total_challenges += 1;
    user.total_staked += stake_amount;

    emit!(ChallengeCreated {
        challenge: ctx.accounts.challenge.key(),
        creator: ctx.accounts.challenge.creator,
        title: title.clone(),
        stake_amount,
        platform_fee,
        start_time,
        end_time,
        required_proofs,
        required_approvals,
        verifier_count: verifiers.len() as u8,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct CreateChallenge<'info> {
    #[account(
        init,
        payer = creator,
        space = 8 + Challenge::INIT_SPACE,
        seeds = [CHALLENGE_SEED, creator.key().as_ref(), &user.total_challenges.to_le_bytes()],
        bump
    )]
    pub challenge: Account<'info, Challenge>,
    #[account(
        mut,
        seeds = [ESCROW_SEED, challenge.key().as_ref()],
        bump
    )]
    /// CHECK: Escrow PDA to hold funds
    pub escrow: AccountInfo<'info>,
    #[account(
        mut,
        seeds = [USER_SEED, creator.key().as_ref()],
        bump = user.bump,
    )]
    pub user: Account<'info, User>,
    #[account(mut)]
    pub creator: Signer<'info>,
    pub system_program: Program<'info, System>,
}
