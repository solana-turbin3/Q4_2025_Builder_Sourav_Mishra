use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

use crate::constants::*;
use crate::errors::ErrorCode;
use crate::events::*;
use crate::state::*;

/// Claim funds after challenge finalization
pub fn claim_funds(ctx: Context<ClaimFunds>) -> Result<()> {
    let clock = Clock::get()?;
    let challenge = &mut ctx.accounts.challenge;
    let challenge_key = challenge.key();
    let claimer = ctx.accounts.claimer.key();

    require!(
        challenge.status == ChallengeStatus::Completed || challenge.status == ChallengeStatus::Failed,
        ErrorCode::InvalidChallengeStatus
    );
    require!(
        clock.unix_timestamp > challenge.finalized_at + DISPUTE_WINDOW,
        ErrorCode::DisputeWindowNotExpired
    );

    match challenge.status {
        ChallengeStatus::Completed => {
            // SUCCESS PATH
            require!(claimer == challenge.creator, ErrorCode::UnauthorizedCreator);
            require!(!challenge.claimed, ErrorCode::AlreadyClaimed);

            let stake_amount = challenge.stake_amount;
            let bonus = (stake_amount * CREATOR_BONUS_BPS) / BASIS_POINTS;
            let creator_reward = stake_amount + bonus;
            let platform_fee_remaining = (stake_amount * PLATFORM_FEE_BPS) / BASIS_POINTS - bonus;

            // Transfer reward to creator
            let escrow_seeds = &[
                ESCROW_SEED,
                challenge_key.as_ref(),
                &[challenge.escrow_bump],
            ];
            let signer_seeds = &[&escrow_seeds[..]];

            transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.system_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.escrow.to_account_info(),
                        to: ctx.accounts.claimer.to_account_info(),
                    },
                    signer_seeds,
                ),
                creator_reward,
            )?;

            // Transfer remaining platform fee to platform
            transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.system_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.escrow.to_account_info(),
                        to: ctx.accounts.platform.to_account_info(),
                    },
                    signer_seeds,
                ),
                platform_fee_remaining,
            )?;

            challenge.claimed = true;
            let user = &mut ctx.accounts.user;
            user.completed += 1;

            emit!(FundsClaimed {
                challenge: challenge_key,
                claimer,
                amount: creator_reward,
                platform_fee: platform_fee_remaining,
                challenge_status: "Completed".to_string(),
                timestamp: clock.unix_timestamp,
            });
        }
        ChallengeStatus::Failed => {
            // FAILURE PATH

            // Check if verification account exists
            let verification = match ctx.accounts.verification.as_ref() {
                Some(v) => v,
                None => return Err(ErrorCode::UnauthorizedVerifier.into()),
            };

            // Validate verification PDA
            let (expected_verification_pda, _bump) = Pubkey::find_program_address(
                &[
                    VERIFICATION_SEED,
                    challenge_key.as_ref(),
                    claimer.as_ref(),
                ],
                &crate::ID,
            );
            require!(
                verification.key() == expected_verification_pda,
                ErrorCode::UnauthorizedVerifier
            );

            // Verify claimer voted REJECT
            require!(verification.vote == Vote::Reject, ErrorCode::VerifierDidNotReject);
            require!(!verification.claimed, ErrorCode::AlreadyClaimed);

            let stake_amount = challenge.stake_amount;
            let rejection_count = challenge.rejection_count;

            // Calculate slashed amount and verifier share
            let slashed_amount = (stake_amount * SLASH_PENALTY_BPS) / BASIS_POINTS;
            let share = slashed_amount / (rejection_count as u64);

            // Transfer share to verifier
            let escrow_seeds = &[
                ESCROW_SEED,
                challenge_key.as_ref(),
                &[challenge.escrow_bump],
            ];
            let signer_seeds = &[&escrow_seeds[..]];

            transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.system_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.escrow.to_account_info(),
                        to: ctx.accounts.claimer.to_account_info(),
                    },
                    signer_seeds,
                ),
                share,
            )?;

            // If this is the first claim, transfer platform fee and creator refund
            if !challenge.claimed {
                let platform_fee = (stake_amount * PLATFORM_FEE_BPS) / BASIS_POINTS;
                let creator_refund = stake_amount - slashed_amount;

                transfer(
                    CpiContext::new_with_signer(
                        ctx.accounts.system_program.to_account_info(),
                        Transfer {
                            from: ctx.accounts.escrow.to_account_info(),
                            to: ctx.accounts.platform.to_account_info(),
                        },
                        signer_seeds,
                    ),
                    platform_fee,
                )?;

                // Transfer creator refund to creator wallet
                transfer(
                    CpiContext::new_with_signer(
                        ctx.accounts.system_program.to_account_info(),
                        Transfer {
                            from: ctx.accounts.escrow.to_account_info(),
                            to: ctx.accounts.creator.to_account_info(),
                        },
                        signer_seeds,
                    ),
                    creator_refund,
                )?;

                challenge.claimed = true;

                // Update user.failed only on first claim
                let user = &mut ctx.accounts.user;
                user.failed += 1;
            }

            // Mark this verification as claimed
            let verification_mut = match ctx.accounts.verification.as_mut() {
                Some(v) => v,
                None => return Err(ErrorCode::UnauthorizedVerifier.into()),
            };
            verification_mut.claimed = true;

            emit!(FundsClaimed {
                challenge: challenge_key,
                claimer,
                amount: share,
                platform_fee: 0,
                challenge_status: "Failed".to_string(),
                timestamp: clock.unix_timestamp,
            });
        }
        _ => {
            return Err(ErrorCode::InvalidChallengeStatus.into());
        }
    }

    Ok(())
}

#[derive(Accounts)]
pub struct ClaimFunds<'info> {
    #[account(mut)]
    pub challenge: Account<'info, Challenge>,
    #[account(
        mut,
        seeds = [ESCROW_SEED, challenge.key().as_ref()],
        bump = challenge.escrow_bump,
    )]
    /// CHECK: Escrow PDA
    pub escrow: AccountInfo<'info>,
    #[account(
        mut,
        seeds = [USER_SEED, challenge.creator.as_ref()],
        bump = user.bump,
    )]
    pub user: Account<'info, User>,
    #[account(mut)]
    /// CHECK: Creator wallet (receives refund on failed challenges)
    pub creator: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK: Platform wallet (receives fees)
    pub platform: AccountInfo<'info>,
    #[account(mut)]
    pub claimer: Signer<'info>,
    // Verification account required for failed challenges only
    /// CHECK: Manually validated in instruction logic
    #[account(mut)]
    pub verification: Option<Account<'info, Verification>>,
    pub system_program: Program<'info, System>,
}
