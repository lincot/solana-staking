import * as anchor from "@project-serum/anchor";
import { BN, Program } from "@project-serum/anchor";
import {
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";
import {
  createAccount,
  createMint,
  getAccount,
  mintTo,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { StakingFactory } from "../target/types/staking_factory";
import { expect } from "chai";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";

chai.use(chaiAsPromised);

function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

describe("staking", () => {
  const connection = new Connection("http://localhost:8899", "recent");
  const stakingFactory = anchor.workspace.StakingFactory as Program<
    StakingFactory
  >;

  const beneficiary = new Keypair();
  const payer = new Keypair();

  it("airdrops", async () => {
    await connection.confirmTransaction(
      await connection.requestAirdrop(
        payer.publicKey,
        100_000_000,
      ),
    );

    await connection.confirmTransaction(
      await connection.requestAirdrop(
        beneficiary.publicKey,
        100_000_000,
      ),
    );
  });

  const mintAuthority = new Keypair();

  let mint: PublicKey;
  let beneficiaryDepositor: PublicKey;

  it("creates mint", async () => {
    mint = await createMint(
      connection,
      payer,
      mintAuthority.publicKey,
      undefined,
      2,
    );

    beneficiaryDepositor = await createAccount(
      connection,
      payer,
      mint,
      beneficiary.publicKey,
    );
    await mintTo(
      connection,
      payer,
      mint,
      beneficiaryDepositor,
      mintAuthority,
      1000,
    );
  });

  const factory = new Keypair();

  it("initializes", async () => {
    await stakingFactory.methods.initialize().accounts({
      factory: factory.publicKey,
      payer: payer.publicKey,
      systemProgram: SystemProgram.programId,
    }).signers([payer, factory]).rpc();
  });

  const staking = new Keypair();
  let stakingSigner: PublicKey;
  let stakingSignerNonce: number;
  const rewardVault = new Keypair();

  it("creates staking", async () => {
    [stakingSigner, stakingSignerNonce] = await PublicKey
      .findProgramAddress(
        [staking.publicKey.toBuffer()],
        stakingFactory.programId,
      );

    await createAccount(connection, payer, mint, stakingSigner, rewardVault);
    await mintTo(
      connection,
      payer,
      mint,
      rewardVault.publicKey,
      mintAuthority,
      1000000,
    );

    await stakingFactory.methods.newStaking(
      stakingSignerNonce,
      mint,
      new BN(2),
      new BN(3600),
      0,
      new BN(1337),
    ).accounts({
      staking: staking.publicKey,
      stakingSigner,
      rewardVault: rewardVault.publicKey,
      payer: payer.publicKey,
      systemProgram: SystemProgram.programId,
    }).signers([payer, staking]).rpc();
  });

  it("changes config", async () => {
    await stakingFactory.methods.changeConfig(new BN(1700), null).accounts({
      staking: staking.publicKey,
    }).signers([staking]).rpc();
  });

  const member = new Keypair();
  let memberSigner: PublicKey;
  let memberSignerNonce: number;
  const available = new Keypair();
  const stake = new Keypair();
  const pending = new Keypair();

  it("creates member", async () => {
    [memberSigner, memberSignerNonce] = await PublicKey.findProgramAddress(
      [staking.publicKey.toBuffer(), member.publicKey.toBuffer()],
      stakingFactory.programId,
    );

    await stakingFactory.methods.createMember(
      memberSignerNonce,
    ).accounts({
      staking: staking.publicKey,
      mint,
      member: member.publicKey,
      beneficiary: beneficiary.publicKey,
      available: available.publicKey,
      stake: stake.publicKey,
      pending: pending.publicKey,
      memberSigner,
      rent: SYSVAR_RENT_PUBKEY,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    }).signers([beneficiary, member, available, stake, pending]).rpc();
  });

  it("deposits", async () => {
    const amount = 120;

    await stakingFactory.methods.deposit(new BN(amount)).accounts({
      member: member.publicKey,
      beneficiary: beneficiary.publicKey,
      available: available.publicKey,
      depositor: beneficiaryDepositor,
      tokenProgram: TOKEN_PROGRAM_ID,
    }).signers([beneficiary]).rpc();

    const availableAccount = await getAccount(
      connection,
      available.publicKey,
    );

    expect(availableAccount.amount).to.eql(BigInt(amount));
  });

  it("stakes", async () => {
    const amount = 10;

    await stakingFactory.methods.stake(new BN(amount)).accounts({
      staking: staking.publicKey,
      member: member.publicKey,
      beneficiary: beneficiary.publicKey,
      available: available.publicKey,
      stake: stake.publicKey,
      memberSigner,
      stakingSigner,
      tokenProgram: TOKEN_PROGRAM_ID,
    }).signers([beneficiary]).rpc();

    const [availableAccount, stakeAccount] = await Promise.all(
      [available, stake].map((v) =>
        getAccount(
          connection,
          v.publicKey,
        )
      ),
    );

    expect(availableAccount.amount).to.eql(BigInt(110));
    expect(stakeAccount.amount).to.eql(BigInt(10));
  });

  it("claims reward", async () => {
    const amount_before = (await getAccount(
      connection,
      beneficiaryDepositor,
    )).amount;

    await stakingFactory.methods.claimReward().accounts({
      staking: staking.publicKey,
      member: member.publicKey,
      beneficiary: beneficiary.publicKey,
      stake: stake.publicKey,
      rewardVault: rewardVault.publicKey,
      to: beneficiaryDepositor,
      stakingSigner,
      tokenProgram: TOKEN_PROGRAM_ID,
    }).signers([beneficiary]).rpc();

    const amount_after = (await getAccount(
      connection,
      beneficiaryDepositor,
    )).amount;

    expect(amount_after - amount_before).to.eq(BigInt(170));
  });

  const pendingWithdrawal = new Keypair();

  it("starts unstake", async () => {
    const amount = 10;

    await stakingFactory.methods.startUnstake(new BN(amount)).accounts({
      staking: staking.publicKey,
      pendingWithdrawal: pendingWithdrawal.publicKey,
      member: member.publicKey,
      beneficiary: beneficiary.publicKey,
      stake: stake.publicKey,
      pending: pending.publicKey,
      memberSigner,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    }).signers([beneficiary, pendingWithdrawal]).rpc();

    const [stakeAccount, pendingAccount] = await Promise.all(
      [stake, pending].map((v) =>
        getAccount(
          connection,
          v.publicKey,
        )
      ),
    );

    expect(stakeAccount.amount).to.eql(BigInt(0));
    expect(pendingAccount.amount).to.eql(BigInt(10));
  });

  const endUnstake = async () => {
    await stakingFactory.methods.endUnstake().accounts({
      staking: staking.publicKey,
      member: member.publicKey,
      beneficiary: beneficiary.publicKey,
      pendingWithdrawal: pendingWithdrawal.publicKey,
      available: available.publicKey,
      pending: pending.publicKey,
      memberSigner,
      tokenProgram: TOKEN_PROGRAM_ID,
    }).signers([beneficiary]).rpc();
  };

  it("fails to end unstake before timelock", async () => {
    await expect(endUnstake()).to.be.rejected;
  });

  it("waits for unstake timelock to end", async () => {
    await sleep(2000);
  });

  it("ends unstake", async () => {
    const amount_before = (await getAccount(
      connection,
      available.publicKey,
    )).amount;

    await endUnstake();

    const amount_after = (await getAccount(
      connection,
      available.publicKey,
    )).amount;

    expect(amount_after - amount_before).to.eq(BigInt(10));
  });

  it("withdraws", async () => {
    const amount = 100;

    const amount_before = (await getAccount(
      connection,
      beneficiaryDepositor,
    )).amount;

    await stakingFactory.methods.withdraw(new BN(amount)).accounts({
      staking: staking.publicKey,
      member: member.publicKey,
      beneficiary: beneficiary.publicKey,
      available: available.publicKey,
      memberSigner,
      receiver: beneficiaryDepositor,
      tokenProgram: TOKEN_PROGRAM_ID,
    }).signers([beneficiary]).rpc();

    const amount_after = (await getAccount(
      connection,
      beneficiaryDepositor,
    )).amount;

    expect(amount_after - amount_before).to.eq(BigInt(amount));
  });
});
