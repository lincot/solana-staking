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
  createMint,
  getAccount,
  getOrCreateAssociatedTokenAccount,
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

  const mintAuthority = new Keypair();
  const factoryAuthority = new Keypair();
  const stakingAuthority = new Keypair();
  const beneficiary = new Keypair();

  it("airdrops", async () => {
    await Promise.all([
      connection.confirmTransaction(
        await connection.requestAirdrop(
          mintAuthority.publicKey,
          100_000_000,
        ),
      ),
      connection.confirmTransaction(
        await connection.requestAirdrop(
          factoryAuthority.publicKey,
          100_000_000,
        ),
      ),
      connection.confirmTransaction(
        await connection.requestAirdrop(
          stakingAuthority.publicKey,
          100_000_000,
        ),
      ),
      connection.confirmTransaction(
        await connection.requestAirdrop(
          beneficiary.publicKey,
          100_000_000,
        ),
      ),
    ]);
  });

  let mint: PublicKey;
  let beneficiaryDepositor: PublicKey;

  it("creates mint", async () => {
    mint = await createMint(
      connection,
      mintAuthority,
      mintAuthority.publicKey,
      undefined,
      2,
    );

    beneficiaryDepositor = (await getOrCreateAssociatedTokenAccount(
      connection,
      beneficiary,
      mint,
      beneficiary.publicKey,
    )).address;
    await mintTo(
      connection,
      beneficiary,
      mint,
      beneficiaryDepositor,
      mintAuthority,
      1000,
    );
  });

  let factory: PublicKey;

  it("initializes", async () => {
    [factory] = await PublicKey
      .findProgramAddress(
        [Buffer.from("factory")],
        stakingFactory.programId,
      );

    await stakingFactory.methods.initialize().accounts({
      factory: factory,
      authority: factoryAuthority.publicKey,
      systemProgram: SystemProgram.programId,
    }).signers([factoryAuthority]).rpc();
  });

  let staking: PublicKey;
  let stakingNonce: number;
  let rewardVault: PublicKey;

  it("creates staking", async () => {
    [staking, stakingNonce] = await PublicKey
      .findProgramAddress(
        [Buffer.from("staking"), new BN(0).toArrayLike(Buffer, "le", 2)],
        stakingFactory.programId,
      );

    [rewardVault] = await PublicKey
      .findProgramAddress(
        [Buffer.from("reward_vault"), staking.toBuffer()],
        stakingFactory.programId,
      );

    await stakingFactory.methods.createStaking(
      stakingNonce,
      mint,
      new BN(2),
      new BN(3600),
      { absolute: {} },
      new BN(1337),
    ).accounts({
      factory,
      staking,
      rewardVault,
      rewardMint: mint,
      authority: stakingAuthority.publicKey,
      rent: SYSVAR_RENT_PUBKEY,
      systemProgram: SystemProgram.programId,
    }).signers([stakingAuthority]).rpc();

    await mintTo(
      connection,
      stakingAuthority,
      mint,
      rewardVault,
      mintAuthority,
      1000000,
    );
  });

  it("changes config", async () => {
    await stakingFactory.methods.changeConfig(new BN(1700), null).accounts({
      staking,
      authority: stakingAuthority.publicKey,
    }).signers([stakingAuthority]).rpc();
  });

  let member: PublicKey;
  let memberNonce: number;
  let available: PublicKey;
  let stake: PublicKey;
  let pending: PublicKey;

  it("creates member", async () => {
    [member, memberNonce] = await PublicKey.findProgramAddress(
      [
        Buffer.from("member"),
        new BN(0).toArrayLike(Buffer, "le", 2),
        beneficiary.publicKey.toBuffer(),
      ],
      stakingFactory.programId,
    );

    [available] = await PublicKey.findProgramAddress(
      [
        Buffer.from("available"),
        member.toBuffer(),
      ],
      stakingFactory.programId,
    );
    [stake] = await PublicKey.findProgramAddress(
      [
        Buffer.from("stake"),
        member.toBuffer(),
      ],
      stakingFactory.programId,
    );
    [pending] = await PublicKey.findProgramAddress(
      [
        Buffer.from("pending"),
        member.toBuffer(),
      ],
      stakingFactory.programId,
    );

    await stakingFactory.methods.createMember(
      memberNonce,
    ).accounts({
      staking,
      mint,
      member,
      beneficiary: beneficiary.publicKey,
      available,
      stake,
      pending,
      rent: SYSVAR_RENT_PUBKEY,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    }).signers([beneficiary]).rpc();
  });

  it("deposits", async () => {
    const amount = 120;

    await stakingFactory.methods.deposit(new BN(amount)).accounts({
      staking,
      member,
      beneficiary: beneficiary.publicKey,
      available,
      depositor: beneficiaryDepositor,
      tokenProgram: TOKEN_PROGRAM_ID,
    }).signers([beneficiary]).rpc();

    const availableAccount = await getAccount(
      connection,
      available,
    );

    expect(availableAccount.amount).to.eql(BigInt(amount));
  });

  it("stakes", async () => {
    const amount = 10;

    await stakingFactory.methods.stake(new BN(amount)).accounts({
      staking,
      member,
      beneficiary: beneficiary.publicKey,
      available,
      stake,
      tokenProgram: TOKEN_PROGRAM_ID,
    }).signers([beneficiary]).rpc();

    const [availableAccount, stakeAccount] = await Promise.all(
      [available, stake].map((v) =>
        getAccount(
          connection,
          v,
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
      staking,
      member,
      beneficiary: beneficiary.publicKey,
      stake,
      rewardVault,
      to: beneficiaryDepositor,
      tokenProgram: TOKEN_PROGRAM_ID,
    }).signers([beneficiary]).rpc();

    const amount_after = (await getAccount(
      connection,
      beneficiaryDepositor,
    )).amount;

    expect(amount_after - amount_before).to.eq(BigInt(170));
  });

  let pendingWithdrawal: PublicKey;

  it("starts unstake", async () => {
    const amount = 10;

    [pendingWithdrawal] = await PublicKey
      .findProgramAddress(
        [Buffer.from("pending_withdrawal"), member.toBuffer()],
        stakingFactory.programId,
      );

    await stakingFactory.methods.startUnstake(new BN(amount)).accounts({
      staking,
      pendingWithdrawal,
      member,
      beneficiary: beneficiary.publicKey,
      stake,
      pending,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    }).signers([beneficiary]).rpc();

    const [stakeAccount, pendingAccount] = await Promise.all(
      [stake, pending].map((v) =>
        getAccount(
          connection,
          v,
        )
      ),
    );

    expect(stakeAccount.amount).to.eql(BigInt(0));
    expect(pendingAccount.amount).to.eql(BigInt(10));
  });

  const endUnstake = async () => {
    await stakingFactory.methods.endUnstake().accounts({
      staking,
      member,
      beneficiary: beneficiary.publicKey,
      pendingWithdrawal,
      available,
      pending,
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
      available,
    )).amount;

    await endUnstake();

    const amount_after = (await getAccount(
      connection,
      available,
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
      staking,
      member,
      beneficiary: beneficiary.publicKey,
      available,
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
