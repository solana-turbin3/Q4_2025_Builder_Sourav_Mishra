import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { ProofOfTouchGrass } from "../target/types/proof_of_touch_grass";
import { PublicKey, Keypair, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { assert } from "chai";
import * as fs from "fs";
import * as os from "os";

describe("Proof of Touch Grass Tests", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.ProofOfTouchGrass as Program<ProofOfTouchGrass>;

  const adminKeypairPath = `${os.homedir()}/.config/solana/id.json`;
  const adminSecretKey = Uint8Array.from(JSON.parse(fs.readFileSync(adminKeypairPath, 'utf-8')));
  const admin = Keypair.fromSecretKey(adminSecretKey);

  let creator: Keypair;
  let verifier1: Keypair;
  let verifier2: Keypair;
  let verifier3: Keypair;
  let platform: Keypair;

  const STAKE_AMOUNT = new BN(10 * LAMPORTS_PER_SOL);
  const DISPUTE_WINDOW = 5;

  async function airdrop(pubkey: PublicKey, amount = 100 * LAMPORTS_PER_SOL) {
    const sig = await provider.connection.requestAirdrop(pubkey, amount);
    await provider.connection.confirmTransaction(sig);
  }

  async function sleep(seconds: number) {
    return new Promise(resolve => setTimeout(resolve, seconds * 1000));
  }

  function getUserPda(authority: PublicKey): PublicKey {
    const [pda] = PublicKey.findProgramAddressSync(
      [Buffer.from("user"), authority.toBuffer()],
      program.programId
    );
    return pda;
  }

  async function getNextChallengePda(authority: PublicKey): Promise<PublicKey> {
    const userPda = getUserPda(authority);
    const user = await program.account.user.fetch(userPda);
    const [challengePda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("challenge"),
        authority.toBuffer(),
        new BN(user.totalChallenges).toArrayLike(Buffer, "le", 4),
      ],
      program.programId
    );
    return challengePda;
  }

  function getEscrowPda(challenge: PublicKey): PublicKey {
    const [pda] = PublicKey.findProgramAddressSync(
      [Buffer.from("escrow"), challenge.toBuffer()],
      program.programId
    );
    return pda;
  }

  function getEvidencePda(challenge: PublicKey, evidenceIndex: number): PublicKey {
    const [pda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("evidence"),
        challenge.toBuffer(),
        new Uint8Array([evidenceIndex]),
      ],
      program.programId
    );
    return pda;
  }

  function getVerificationPda(challenge: PublicKey, verifier: PublicKey): PublicKey {
    const [pda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("verification"),
        challenge.toBuffer(),
        verifier.toBuffer(),
      ],
      program.programId
    );
    return pda;
  }

  function getDisputePda(challenge: PublicKey): PublicKey {
    const [pda] = PublicKey.findProgramAddressSync(
      [Buffer.from("dispute"), challenge.toBuffer()],
      program.programId
    );
    return pda;
  }

  async function createChallenge(
    creatorKeypair: Keypair,
    options: {
      title?: string;
      description?: string;
      stakeAmount?: BN;
      startTime?: BN;
      endTime?: BN;
      verificationPeriod?: BN;
      requiredProofs?: number;
      requiredApprovals?: number;
      verifiers?: PublicKey[];
    } = {}
  ): Promise<PublicKey> {
    const now = Math.floor(Date.now() / 1000);
    const defaults = {
      title: "Test Challenge",
      description: "Test description",
      stakeAmount: STAKE_AMOUNT,
      startTime: new BN(now - 100),
      endTime: new BN(now + 3600),
      verificationPeriod: new BN(300),
      requiredProofs: 2,
      requiredApprovals: 2,
      verifiers: [verifier1.publicKey, verifier2.publicKey],
    };
    const params = { ...defaults, ...options };

    const userPda = getUserPda(creatorKeypair.publicKey);
    const challengePda = await getNextChallengePda(creatorKeypair.publicKey);
    const escrowPda = getEscrowPda(challengePda);

    await program.methods
      .createChallenge(
        params.title,
        params.description,
        params.stakeAmount,
        params.startTime,
        params.endTime,
        params.verificationPeriod,
        params.requiredProofs,
        params.requiredApprovals,
        params.verifiers
      )
      .accounts({
        challenge: challengePda,
        escrow: escrowPda,
        user: userPda,
        creator: creatorKeypair.publicKey,
      })
      .signers([creatorKeypair])
      .rpc();

    return challengePda;
  }

  async function activateChallenge(challenge: PublicKey, creatorPubkey: PublicKey): Promise<void> {
    const userPda = getUserPda(creatorPubkey);
    await program.methods
      .updateChallengeState()
      .accounts({
        challenge,
        user: userPda,
        admin: admin.publicKey,
      })
      .signers([admin])
      .rpc();
  }

  async function submitEvidence(
    challenge: PublicKey,
    submitter: Keypair,
    evidenceIndex: number,
    ipfsHash: string = "QmTestHash",
    metadata: string = "Test evidence"
  ): Promise<PublicKey> {
    const evidencePda = getEvidencePda(challenge, evidenceIndex);
    await program.methods
      .submitEvidence(ipfsHash, metadata)
      .accounts({
        evidence: evidencePda,
        challenge,
        submitter: submitter.publicKey,
      })
      .signers([submitter])
      .rpc();
    return evidencePda;
  }

  async function verifyEvidence(
    challenge: PublicKey,
    verifier: Keypair,
    creatorPubkey: PublicKey,
    approve: boolean = true
  ): Promise<PublicKey> {
    const verificationPda = getVerificationPda(challenge, verifier.publicKey);
    const userPda = getUserPda(creatorPubkey);
    await program.methods
      .verifyEvidence(approve ? { approve: {} } : { reject: {} })
      .accounts({
        verification: verificationPda,
        challenge,
        user: userPda,
        verifier: verifier.publicKey,
      })
      .signers([verifier])
      .rpc();
    return verificationPda;
  }

  before(async () => {
    creator = Keypair.generate();
    verifier1 = Keypair.generate();
    verifier2 = Keypair.generate();
    verifier3 = Keypair.generate();
    platform = Keypair.generate();

    await Promise.all([
      airdrop(creator.publicKey),
      airdrop(verifier1.publicKey),
      airdrop(verifier2.publicKey),
      airdrop(verifier3.publicKey),
      airdrop(admin.publicKey),
    ]);
  });

  describe("1. Initialize User", () => {
    let userPda: PublicKey;

    it("Successfully creates user profile", async () => {
      [userPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("user"), creator.publicKey.toBuffer()],
        program.programId
      );

      await program.methods
        .initializeUser()
        .accounts({
          user: userPda,
          authority: creator.publicKey,
        })
        .signers([creator])
        .rpc();

      const user = await program.account.user.fetch(userPda);
      assert.equal(user.authority.toString(), creator.publicKey.toString());
      assert.equal(user.totalChallenges, 0);
      assert.equal(user.completed, 0);
      assert.equal(user.failed, 0);
      assert.equal(user.totalStaked.toNumber(), 0);
    });

    it("Fails to initialize user twice", async () => {
      try {
        await program.methods
          .initializeUser()
          .accounts({
            user: userPda,
            authority: creator.publicKey,
          })
          .signers([creator])
          .rpc();
        assert.fail("Should have failed");
      } catch (err) {
        assert.ok(err);
      }
    });
  });

  describe("2. Create Challenge", () => {
    it("Successfully creates challenge", async () => {
      const now = Math.floor(Date.now() / 1000);
      const challengePda = await createChallenge(creator, {
        title: "30-Day Walk Challenge",
        description: "Walk 30 minutes daily for 30 days",
        startTime: new BN(now + 10),
        verifiers: [verifier1.publicKey, verifier2.publicKey, verifier3.publicKey],
      });

      const challenge = await program.account.challenge.fetch(challengePda);
      assert.equal(challenge.creator.toString(), creator.publicKey.toString());
      assert.equal(challenge.stakeAmount.toNumber(), STAKE_AMOUNT.toNumber());
      assert.equal(challenge.evidenceCount, 0);
      assert.equal(challenge.approvalCount, 0);
      assert.equal(challenge.rejectionCount, 0);
      assert.equal(challenge.claimed, false);
      assert.ok(challenge.status.created);

      const userPda = getUserPda(creator.publicKey);
      const user = await program.account.user.fetch(userPda);
      assert.equal(user.totalChallenges, 1);
    });

    it("Fails with invalid stake amount", async () => {
      try {
        await createChallenge(creator, {
          title: "Bad Challenge",
          description: "Invalid stake",
          stakeAmount: new BN(0),
          requiredProofs: 1,
          requiredApprovals: 1,
          verifiers: [verifier1.publicKey],
        });
        assert.fail("Should have failed");
      } catch (err) {
        assert.include(err.toString(), "InvalidStakeAmount");
      }
    });

    it("Fails with invalid time range", async () => {
      const now = Math.floor(Date.now() / 1000);
      try {
        await createChallenge(creator, {
          title: "Bad Time Challenge",
          description: "End before start",
          startTime: new BN(now + 3600),
          endTime: new BN(now + 10),
          requiredProofs: 1,
          requiredApprovals: 1,
          verifiers: [verifier1.publicKey],
        });
        assert.fail("Should have failed");
      } catch (err) {
        assert.include(err.toString(), "InvalidTimeRange");
      }
    });
  });

  describe("3. Update Challenge State (Admin)", () => {
    it("Transitions Created → Active when start_time reached", async () => {
      const challengePda = await createChallenge(creator, {
        title: "State Update Test",
        description: "Testing state transitions",
      });

      await activateChallenge(challengePda, creator.publicKey);

      const challenge = await program.account.challenge.fetch(challengePda);
      assert.ok(challenge.status.active);
    });

    it("Rejects non-admin callers", async () => {
      const challengePda = await createChallenge(creator, {
        title: "Admin Auth Test",
      });

      const userPda = getUserPda(creator.publicKey);
      try {
        await program.methods
          .updateChallengeState()
          .accounts({
            challenge: challengePda,
            user: userPda,
            admin: creator.publicKey,
          })
          .signers([creator])
          .rpc();
        assert.fail("Should have failed");
      } catch (err) {
        assert.include(err.toString(), "UnauthorizedAdmin");
      }
    });
  });

  describe("4. Submit Evidence", () => {
    it("Submits first evidence successfully", async () => {
      const challengePda = await createChallenge(creator, {
        title: "Evidence Test",
        description: "Testing evidence submission",
      });
      await activateChallenge(challengePda, creator.publicKey);

      await submitEvidence(
        challengePda,
        creator,
        0,
        "QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG",
        "Day 1 morning walk"
      );

      const challenge = await program.account.challenge.fetch(challengePda);
      assert.equal(challenge.evidenceCount, 1);
      assert.ok(challenge.status.active);
    });

    it("Submits second evidence and transitions to PendingVerification", async () => {
      const challengePda = await createChallenge(creator, {
        title: "Evidence Test 2",
      });
      await activateChallenge(challengePda, creator.publicKey);

      await submitEvidence(challengePda, creator, 0, "QmHash1", "Evidence 1");

      await submitEvidence(challengePda, creator, 1, "QmHash2", "Evidence 2");

      const challenge = await program.account.challenge.fetch(challengePda);
      assert.equal(challenge.evidenceCount, 2);
      assert.ok(challenge.status.pendingVerification);
    });

    it("Rejects evidence from non-creator", async () => {
      const challengePda = await createChallenge(creator, {
        title: "Auth Test",
      });
      await activateChallenge(challengePda, creator.publicKey);

      try {
        await submitEvidence(challengePda, verifier1, 0, "QmBadHash", "Invalid");
        assert.fail("Should have failed");
      } catch (err) {
        assert.include(err.toString(), "UnauthorizedSubmitter");
      }
    });
  });

  describe("5. Verify Evidence (With Early Finalization)", () => {
    before(async () => {
      await airdrop(creator.publicKey);
    });

    it("Verifier 1 approves evidence", async () => {
      const challengePda = await createChallenge(creator, {
        title: "Verification Test",
        verifiers: [verifier1.publicKey, verifier2.publicKey, verifier3.publicKey],
      });

      await activateChallenge(challengePda, creator.publicKey);
      await submitEvidence(challengePda, creator, 0, "QmHash1", "Evidence 1");
      await submitEvidence(challengePda, creator, 1, "QmHash2", "Evidence 2");

      await verifyEvidence(challengePda, verifier1, creator.publicKey, true);

      const challenge = await program.account.challenge.fetch(challengePda);
      assert.equal(challenge.approvalCount, 1);
      assert.ok(challenge.status.pendingVerification);
    });

    it("Verifier 2 approves and triggers early finalization", async () => {
      const challengePda = await createChallenge(creator, {
        title: "Early Finalization Test",
        verifiers: [verifier1.publicKey, verifier2.publicKey, verifier3.publicKey],
      });

      await activateChallenge(challengePda, creator.publicKey);
      await submitEvidence(challengePda, creator, 0, "QmHash1", "Evidence 1");
      await submitEvidence(challengePda, creator, 1, "QmHash2", "Evidence 2");

      await verifyEvidence(challengePda, verifier1, creator.publicKey, true);

      await verifyEvidence(challengePda, verifier2, creator.publicKey, true);

      const challenge = await program.account.challenge.fetch(challengePda);
      assert.equal(challenge.approvalCount, 2);
      assert.ok(challenge.status.completed);
      assert.isAbove(challenge.finalizedAt.toNumber(), 0);

      const userPda = getUserPda(creator.publicKey);
      const user = await program.account.user.fetch(userPda);
      assert.isAtLeast(user.completed, 1);
    });

    it("Rejects vote from non-verifier", async () => {
      const challengePda = await createChallenge(creator, {
        title: "Verifier Auth Test",
        verifiers: [verifier1.publicKey, verifier2.publicKey],
      });

      await activateChallenge(challengePda, creator.publicKey);
      await submitEvidence(challengePda, creator, 0, "QmHash", "Evidence");
      await submitEvidence(challengePda, creator, 1, "QmHash2", "Evidence2");

      const randomUser = Keypair.generate();
      await airdrop(randomUser.publicKey);

      try {
        await verifyEvidence(challengePda, randomUser, creator.publicKey, true);
        assert.fail("Should have failed");
      } catch (err) {
        assert.include(err.toString(), "UnauthorizedVerifier");
      }
    });

    it("Verifiers reject evidence (negative votes)", async () => {
      const challengePda = await createChallenge(creator, {
        title: "Rejection Test",
        requiredProofs: 2,
        requiredApprovals: 2,
        verifiers: [verifier1.publicKey, verifier2.publicKey, verifier3.publicKey],
      });

      await activateChallenge(challengePda, creator.publicKey);
      await submitEvidence(challengePda, creator, 0, "QmHash1", "Evidence 1");
      await submitEvidence(challengePda, creator, 1, "QmHash2", "Evidence 2");

      await verifyEvidence(challengePda, verifier1, creator.publicKey, false);
      await verifyEvidence(challengePda, verifier2, creator.publicKey, false);

      const challenge = await program.account.challenge.fetch(challengePda);
      assert.equal(challenge.approvalCount, 0);
      assert.equal(challenge.rejectionCount, 2);
      assert.ok(challenge.status.failed);

      const userPda = getUserPda(creator.publicKey);
      const user = await program.account.user.fetch(userPda);
      assert.isAtLeast(user.failed, 1);
    });

    it("Insufficient approvals - threshold not reached", async () => {
      const challengePda = await createChallenge(creator, {
        title: "Threshold Test",
        requiredProofs: 2,
        requiredApprovals: 3,
        verifiers: [verifier1.publicKey, verifier2.publicKey, verifier3.publicKey],
      });

      await activateChallenge(challengePda, creator.publicKey);
      await submitEvidence(challengePda, creator, 0, "QmHash1", "Evidence 1");
      await submitEvidence(challengePda, creator, 1, "QmHash2", "Evidence 2");

      await verifyEvidence(challengePda, verifier1, creator.publicKey, true);
      await verifyEvidence(challengePda, verifier2, creator.publicKey, true);

      const challenge = await program.account.challenge.fetch(challengePda);
      assert.equal(challenge.approvalCount, 2);
      assert.ok(challenge.status.pendingVerification);
    });

    it("No verifiers vote - innocent until proven guilty", async () => {
      const now = Math.floor(Date.now() / 1000);
      const challengePda = await createChallenge(creator, {
        title: "No Vote Test",
        startTime: new BN(now - 100),
        endTime: new BN(now + 2),
        verificationPeriod: new BN(3),
        requiredProofs: 2,
        requiredApprovals: 2,
        verifiers: [verifier1.publicKey, verifier2.publicKey],
      });

      await activateChallenge(challengePda, creator.publicKey);
      await submitEvidence(challengePda, creator, 0, "QmHash1", "Evidence 1");
      await submitEvidence(challengePda, creator, 1, "QmHash2", "Evidence 2");

      let challenge = await program.account.challenge.fetch(challengePda);
      assert.ok(challenge.status.pendingVerification);

      await sleep(6);

      const userPda = getUserPda(creator.publicKey);
      await program.methods
        .updateChallengeState()
        .accounts({
          challenge: challengePda,
          user: userPda,
          admin: admin.publicKey,
        })
        .signers([admin])
        .rpc();

      challenge = await program.account.challenge.fetch(challengePda);
      assert.ok(challenge.status.completed);
      assert.equal(challenge.approvalCount, 0);
      assert.equal(challenge.rejectionCount, 0);

      const user = await program.account.user.fetch(userPda);
      assert.isAtLeast(user.completed, 1);
    });
  });

  describe("6. Claim Funds (Success Path)", () => {
    before(async () => {
      await airdrop(creator.publicKey);
    });

    it("Fails before dispute window expires", async () => {
      const challengePda = await createChallenge(creator, {
        title: "Claim Test",
        requiredProofs: 1,
        requiredApprovals: 1,
        verifiers: [verifier1.publicKey],
      });

      await activateChallenge(challengePda, creator.publicKey);
      await submitEvidence(challengePda, creator, 0);
      await verifyEvidence(challengePda, verifier1, creator.publicKey, true);

      const escrowPda = getEscrowPda(challengePda);
      const userPda = getUserPda(creator.publicKey);

      try {
        await program.methods
          .claimFunds()
          .accounts({
            challenge: challengePda,
            escrow: escrowPda,
            user: userPda,
            creator: creator.publicKey,
            platform: platform.publicKey,
            claimer: creator.publicKey,
            verification: null,
          })
          .signers([creator])
          .rpc();
        assert.fail("Should have failed");
      } catch (err) {
        assert.include(err.toString(), "DisputeWindowNotExpired");
      }
    });

    it("Creator claims stake + bonus after dispute window", async () => {
      const challengePda = await createChallenge(creator, {
        title: "Claim Test 2",
        requiredProofs: 1,
        requiredApprovals: 1,
        verifiers: [verifier1.publicKey],
      });

      await activateChallenge(challengePda, creator.publicKey);
      await submitEvidence(challengePda, creator, 0);
      await verifyEvidence(challengePda, verifier1, creator.publicKey, true);

      await sleep(DISPUTE_WINDOW + 1);

      const escrowPda = getEscrowPda(challengePda);
      const userPda = getUserPda(creator.publicKey);
      const balanceBefore = await provider.connection.getBalance(creator.publicKey);

      await program.methods
        .claimFunds()
        .accounts({
          challenge: challengePda,
          escrow: escrowPda,
          user: userPda,
          creator: creator.publicKey,
          platform: platform.publicKey,
          claimer: creator.publicKey,
          verification: null,
        })
        .signers([creator])
        .rpc();

      const balanceAfter = await provider.connection.getBalance(creator.publicKey);
      const reward = balanceAfter - balanceBefore;

      const expectedReward = 10 * LAMPORTS_PER_SOL + (10 * LAMPORTS_PER_SOL * 250) / 10000;
      assert.approximately(reward, expectedReward, 0.01 * LAMPORTS_PER_SOL);

      const challenge = await program.account.challenge.fetch(challengePda);
      assert.equal(challenge.claimed, true);
    });

    it("Verifiers claim rewards after successful rejection", async () => {
      const challengePda = await createChallenge(creator, {
        title: "Verifier Claim Test",
        requiredProofs: 1,
        requiredApprovals: 2,
        verifiers: [verifier1.publicKey, verifier2.publicKey, verifier3.publicKey],
      });

      await activateChallenge(challengePda, creator.publicKey);
      await submitEvidence(challengePda, creator, 0);

      await verifyEvidence(challengePda, verifier1, creator.publicKey, false);
      await verifyEvidence(challengePda, verifier2, creator.publicKey, false);

      let challenge = await program.account.challenge.fetch(challengePda);
      assert.ok(challenge.status.failed);

      await sleep(DISPUTE_WINDOW + 1);

      const escrowPda = getEscrowPda(challengePda);
      const userPda = getUserPda(creator.publicKey);
      const verificationPda = getVerificationPda(challengePda, verifier1.publicKey);

      const verifier1BalanceBefore = await provider.connection.getBalance(verifier1.publicKey);
      const creatorBalanceBefore = await provider.connection.getBalance(creator.publicKey);
      const platformBalanceBefore = await provider.connection.getBalance(platform.publicKey);

      await program.methods
        .claimFunds()
        .accounts({
          challenge: challengePda,
          escrow: escrowPda,
          user: userPda,
          creator: creator.publicKey,
          platform: platform.publicKey,
          claimer: verifier1.publicKey,
          verification: verificationPda,
        })
        .signers([verifier1])
        .rpc();

      const verifier1BalanceAfter = await provider.connection.getBalance(verifier1.publicKey);
      const creatorBalanceAfter = await provider.connection.getBalance(creator.publicKey);
      const platformBalanceAfter = await provider.connection.getBalance(platform.publicKey);

      const verifier1Reward = verifier1BalanceAfter - verifier1BalanceBefore;
      const creatorRefund = creatorBalanceAfter - creatorBalanceBefore;
      const platformFee = platformBalanceAfter - platformBalanceBefore;

      const expectedVerifierShare = (10 * LAMPORTS_PER_SOL * 2500) / 10000 / 2;
      const expectedCreatorRefund = (10 * LAMPORTS_PER_SOL * 7500) / 10000;
      const expectedPlatformFee = (10 * LAMPORTS_PER_SOL * 500) / 10000;

      assert.approximately(verifier1Reward, expectedVerifierShare, 0.01 * LAMPORTS_PER_SOL);
      assert.approximately(creatorRefund, expectedCreatorRefund, 0.01 * LAMPORTS_PER_SOL);
      assert.approximately(platformFee, expectedPlatformFee, 0.01 * LAMPORTS_PER_SOL);

      challenge = await program.account.challenge.fetch(challengePda);
      assert.equal(challenge.claimed, true);

      const verifier2BalanceBefore = await provider.connection.getBalance(verifier2.publicKey);
      const verificationPda2 = getVerificationPda(challengePda, verifier2.publicKey);

      await program.methods
        .claimFunds()
        .accounts({
          challenge: challengePda,
          escrow: escrowPda,
          user: userPda,
          creator: creator.publicKey,
          platform: platform.publicKey,
          claimer: verifier2.publicKey,
          verification: verificationPda2,
        })
        .signers([verifier2])
        .rpc();

      const verifier2BalanceAfter = await provider.connection.getBalance(verifier2.publicKey);
      const verifier2Reward = verifier2BalanceAfter - verifier2BalanceBefore;

      assert.approximately(verifier2Reward, expectedVerifierShare, 0.01 * LAMPORTS_PER_SOL);
    });

    it("Non-voting verifiers cannot claim rewards", async () => {
      const challengePda = await createChallenge(creator, {
        title: "Non-Voter Test",
        requiredProofs: 1,
        requiredApprovals: 2,
        verifiers: [verifier1.publicKey, verifier2.publicKey, verifier3.publicKey],
      });

      await activateChallenge(challengePda, creator.publicKey);
      await submitEvidence(challengePda, creator, 0);

      await verifyEvidence(challengePda, verifier1, creator.publicKey, false);
      await verifyEvidence(challengePda, verifier2, creator.publicKey, false);

      let challenge = await program.account.challenge.fetch(challengePda);
      assert.ok(challenge.status.failed);

      await sleep(DISPUTE_WINDOW + 1);

      const escrowPda = getEscrowPda(challengePda);
      const userPda = getUserPda(creator.publicKey);

      const verification1Pda = getVerificationPda(challengePda, verifier1.publicKey);
      const creatorBalanceBefore = await provider.connection.getBalance(creator.publicKey);

      await program.methods
        .claimFunds()
        .accounts({
          challenge: challengePda,
          escrow: escrowPda,
          user: userPda,
          creator: creator.publicKey,
          platform: platform.publicKey,
          claimer: verifier1.publicKey,
          verification: verification1Pda,
        })
        .signers([verifier1])
        .rpc();

      const creatorBalanceAfter = await provider.connection.getBalance(creator.publicKey);
      const creatorRefund = creatorBalanceAfter - creatorBalanceBefore;
      const expectedCreatorRefund = (10 * LAMPORTS_PER_SOL * 7500) / 10000;
      assert.approximately(creatorRefund, expectedCreatorRefund, 0.01 * LAMPORTS_PER_SOL);

      const verification3Pda = getVerificationPda(challengePda, verifier3.publicKey);

      try {
        await program.methods
          .claimFunds()
          .accounts({
            challenge: challengePda,
            escrow: escrowPda,
            user: userPda,
            creator: creator.publicKey,
            platform: platform.publicKey,
            claimer: verifier3.publicKey,
            verification: verification3Pda,
          })
          .signers([verifier3])
          .rpc();
        assert.fail("Should have failed - non-voter shouldn't be able to claim");
      } catch (err) {
        assert.ok(err);
      }
    });
  });

  describe("7. Cancel Challenge", () => {
    before(async () => {
      await airdrop(creator.publicKey);
    });

    it("Cancels in Created status with full refund", async () => {
      const now = Math.floor(Date.now() / 1000);
      const challengePda = await createChallenge(creator, {
        title: "Cancel Created Test",
        startTime: new BN(now + 3600),
        endTime: new BN(now + 7200),
        requiredProofs: 1,
        requiredApprovals: 1,
        verifiers: [verifier1.publicKey],
      });

      const escrowPda = getEscrowPda(challengePda);
      const balanceBefore = await provider.connection.getBalance(creator.publicKey);

      await program.methods
        .cancelChallenge()
        .accounts({
          challenge: challengePda,
          escrow: escrowPda,
          platform: platform.publicKey,
          creator: creator.publicKey,
        })
        .signers([creator])
        .rpc();

      const balanceAfter = await provider.connection.getBalance(creator.publicKey);
      const refund = balanceAfter - balanceBefore;

      const expectedRefund = 10.5 * LAMPORTS_PER_SOL;
      assert.approximately(refund, expectedRefund, 0.01 * LAMPORTS_PER_SOL);

      const challenge = await program.account.challenge.fetch(challengePda);
      assert.ok(challenge.status.cancelled);
    });

    it("Cancels in Active status with penalty", async () => {
      const challengePda = await createChallenge(creator, {
        title: "Cancel Active Test",
        requiredProofs: 1,
        requiredApprovals: 1,
        verifiers: [verifier1.publicKey],
      });

      await activateChallenge(challengePda, creator.publicKey);

      const escrowPda = getEscrowPda(challengePda);
      const balanceBefore = await provider.connection.getBalance(creator.publicKey);

      await program.methods
        .cancelChallenge()
        .accounts({
          challenge: challengePda,
          escrow: escrowPda,
          platform: platform.publicKey,
          creator: creator.publicKey,
        })
        .signers([creator])
        .rpc();

      const balanceAfter = await provider.connection.getBalance(creator.publicKey);
      const refund = balanceAfter - balanceBefore;

      const expectedRefund = 10.3 * LAMPORTS_PER_SOL;
      assert.approximately(refund, expectedRefund, 0.01 * LAMPORTS_PER_SOL);

      const challenge = await program.account.challenge.fetch(challengePda);
      assert.ok(challenge.status.cancelled);
    });

    it("Fails to cancel from non-creator", async () => {
      const now = Math.floor(Date.now() / 1000);
      const challengePda = await createChallenge(creator, {
        title: "Cancel Auth Test",
        startTime: new BN(now + 3600),
        endTime: new BN(now + 7200),
        requiredProofs: 1,
        requiredApprovals: 1,
        verifiers: [verifier1.publicKey],
      });

      const escrowPda = getEscrowPda(challengePda);
      try {
        await program.methods
          .cancelChallenge()
          .accounts({
            challenge: challengePda,
            escrow: escrowPda,
            platform: platform.publicKey,
            creator: verifier1.publicKey,
          })
          .signers([verifier1])
          .rpc();
        assert.fail("Should have failed");
      } catch (err) {
        assert.include(err.toString(), "UnauthorizedCreator");
      }
    });
  });

  describe("8. Dispute Verification", () => {
    before(async () => {
      await airdrop(creator.publicKey);
    });

    it("Creator files dispute within window", async () => {
      const challengePda = await createChallenge(creator, {
        title: "Dispute Test",
        requiredProofs: 1,
        requiredApprovals: 1,
        verifiers: [verifier1.publicKey],
      });

      await activateChallenge(challengePda, creator.publicKey);
      await submitEvidence(challengePda, creator, 0);
      await verifyEvidence(challengePda, verifier1, creator.publicKey, true);

      const disputePda = getDisputePda(challengePda);
      await program.methods
        .disputeVerification("Evidence appears fraudulent")
        .accounts({
          dispute: disputePda,
          challenge: challengePda,
          disputer: creator.publicKey,
        })
        .signers([creator])
        .rpc();

      const challenge = await program.account.challenge.fetch(challengePda);
      assert.ok(challenge.status.disputed);

      const dispute = await program.account.dispute.fetch(disputePda);
      assert.equal(dispute.disputer.toString(), creator.publicKey.toString());
      assert.equal(dispute.reason, "Evidence appears fraudulent");
    });

    it("Fails to dispute after window expires", async () => {
      const challengePda = await createChallenge(creator, {
        title: "Dispute Expired Test",
        requiredProofs: 1,
        requiredApprovals: 1,
        verifiers: [verifier1.publicKey],
      });

      await activateChallenge(challengePda, creator.publicKey);
      await submitEvidence(challengePda, creator, 0);
      await verifyEvidence(challengePda, verifier1, creator.publicKey, true);

      await sleep(DISPUTE_WINDOW + 1);

      const disputePda = getDisputePda(challengePda);
      try {
        await program.methods
          .disputeVerification("Too late")
          .accounts({
            dispute: disputePda,
            challenge: challengePda,
            disputer: creator.publicKey,
          })
          .signers([creator])
          .rpc();
        assert.fail("Should have failed");
      } catch (err) {
        assert.include(err.toString(), "DisputeWindowExpired");
      }
    });
  });
});
