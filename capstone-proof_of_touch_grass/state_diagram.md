# Proof of Touch Grass -  State Diagram

```mermaid
stateDiagram-v2
    [*] --> Created: create_challenge()
    Created --> Active: start_time reached
    Created --> Cancelled: cancel_challenge()<br/>(before start_time)
    Active --> Verifying: All required evidence submitted<br/>(evidence_count == required_proofs)
    Active --> Failed: end_time reached<br/>Evidence incomplete
    Active --> Cancelled: cancel_challenge()<br/>(with penalty)
    Verifying --> Completed: Approval threshold met<br/>(approval_count >= required_approvals)<br/>finalize_challenge()
    Verifying --> Failed: Rejection threshold met<br/>(rejection_count > verifiers - required_approvals)<br/>finalize_challenge()
    Verifying --> Failed: verification_period_end reached<br/>Insufficient approvals
    Completed --> Disputed: dispute_verification()<br/>(within dispute window)
    Failed --> Disputed: dispute_verification()<br/>(within dispute window)
    Disputed --> Completed: Dispute resolved<br/>(in favor of creator)
    Disputed --> Failed: Dispute resolved<br/>(in favor of verifiers)
    Completed --> [*]: Stake returned + bonus
    Failed --> [*]: Stake slashed & distributed
    Cancelled --> [*]: Stake refunded (minus fee)

    note right of Created
        Initial state after challenge creation
        - SOL locked in escrow
        - Verifiers assigned
        - Waiting for start_time
    end note
    note right of Active
        Challenge is live
        - Creator can submit evidence
        - Timer running until end_time
        - Min required_proofs needed
    end note
    note right of Verifying
        Evidence submitted, awaiting votes
        - Verifiers vote approve/reject
        - Runs until verification_period_end
        - Needs required_approvals to pass
    end note
    note right of Completed
        Challenge succeeded
        - Creator gets stake back
        - Creator gets bonus (from pool)
        - Reputation increases
    end note
    note right of Failed
        Challenge failed
        - Stake slashed
        - Distributed to active verifiers
        - Reputation decreases
    end note
    note right of Disputed
        Verification contested
        - All funds locked
        - Admin/DAO review needed
        - Can resolve to either outcome
    end note
    note right of Cancelled
        Challenge terminated early
        - Small cancellation fee
        - Remaining stake refunded
        - No reputation impact
    end note
```

## State Transitions Overview

### Challenge States

1. **Created** → Initial state with stake locked in escrow, waiting for start_time
2. **Active** → Challenge is live, creator can submit evidence until end_time
3. **Verifying** → All evidence submitted, verifiers vote within verification period
4. **Completed** → Success! Creator receives stake back + bonus reward
5. **Failed** → Evidence rejected or incomplete, stake slashed to verifiers
6. **Cancelled** → Early termination with small penalty, stake mostly refunded
7. **Disputed** → Contested outcome, all funds locked pending resolution
