import * as anchor from "@project-serum/anchor";
import { BN, Program } from "@project-serum/anchor";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import {
  createAccount,
  createMint,
  getAccount,
  mintTo,
  TOKEN_PROGRAM_ID,
  transfer,
} from "@solana/spl-token";
import { Registry } from "../target/types/registry";
import { expect } from "chai";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";

chai.use(chaiAsPromised);

function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

describe("registry", () => {
  const connection = new Connection("http://localhost:8899", "recent");
  const registry = anchor.workspace.Registry as Program<Registry>;

  const payer = new Keypair();

  it("airdrops", async () => {
    await connection.confirmTransaction(
      await connection.requestAirdrop(
        payer.publicKey,
        100_000_000,
      ),
    );
  });

  const mintAuthority = new Keypair();
  const god = new Keypair();
  const beneficiary = new Keypair();

  let mint: PublicKey;
  let godDepositor: PublicKey;
  let beneficiaryDepositor: PublicKey;

  it("creates mint", async () => {
    mint = await createMint(
      connection,
      payer,
      mintAuthority.publicKey,
      undefined,
      2,
    );

    godDepositor = await createAccount(
      connection,
      payer,
      mint,
      god.publicKey,
    );
    await mintTo(
      connection,
      payer,
      mint,
      godDepositor,
      mintAuthority,
      1000000,
    );

    beneficiaryDepositor = await createAccount(
      connection,
      payer,
      mint,
      beneficiary.publicKey,
    );
    await transfer(
      connection,
      payer,
      godDepositor,
      beneficiaryDepositor,
      god,
      500,
    );
  });

  const registrar = new Keypair();
  let registrarSigner: PublicKey;
  let registrarSignerNonce: number;
  let poolMint: PublicKey;

  it("creates pool mint", async () => {
    [registrarSigner, registrarSignerNonce] = await PublicKey
      .findProgramAddress(
        [registrar.publicKey.toBuffer()],
        registry.programId,
      );
    poolMint = await createMint(
      connection,
      payer,
      registrarSigner,
      undefined,
      0,
    );
  });

  const registryAuthority = new Keypair();
  const rewardQueue = new Keypair();
  const vendorVault = new Keypair();

  it("initializes registry", async () => {
    await createAccount(connection, payer, mint, registrarSigner, vendorVault);

    await registry.methods.initialize(
      registrarSignerNonce,
      mint,
      registryAuthority.publicKey,
      new BN(2),
      new BN(2),
      170,
    ).accounts({
      registrar: registrar.publicKey,
      poolMint,
      rewardQueue: rewardQueue.publicKey,
      vendorVault: vendorVault.publicKey,
      registrarSigner,
    }).signers([registrar, rewardQueue]).preInstructions(
      await Promise.all([
        registry.account.registrar.createInstruction(registrar),
        registry.account.rewardQueue.createInstruction(rewardQueue, 8250),
      ]),
    ).rpc();
  });

  const member = new Keypair();
  let memberSigner: PublicKey;
  let memberSignerNonce: number;
  const spt = new Keypair();
  const available = new Keypair();
  const stake = new Keypair();
  const pending = new Keypair();

  it("creates member", async () => {
    [memberSigner, memberSignerNonce] = await PublicKey.findProgramAddress(
      [registrar.publicKey.toBuffer(), member.publicKey.toBuffer()],
      registry.programId,
    );

    await Promise.all([
      createAccount(connection, payer, poolMint, memberSigner, spt),
      createAccount(connection, payer, mint, memberSigner, available),
      createAccount(connection, payer, mint, memberSigner, stake),
      createAccount(connection, payer, mint, memberSigner, pending),
    ]);

    await registry.methods.createMember(
      memberSignerNonce,
    ).accounts({
      registrar: registrar.publicKey,
      member: member.publicKey,
      beneficiary: beneficiary.publicKey,
      memberSigner,
      spt: spt.publicKey,
      available: available.publicKey,
      stake: stake.publicKey,
      pending: pending.publicKey,
      tokenProgram: TOKEN_PROGRAM_ID,
    }).preInstructions([
      await registry.account.member.createInstruction(member),
    ]).signers([beneficiary, member]).rpc();
  });

  it("deposits", async () => {
    const amount = 120;

    await registry.methods.deposit(new BN(amount)).accounts({
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

    await registry.methods.stake(new BN(amount)).accounts({
      registrar: registrar.publicKey,
      rewardQueue: rewardQueue.publicKey,
      poolMint,
      member: member.publicKey,
      beneficiary: beneficiary.publicKey,
      spt: spt.publicKey,
      available: available.publicKey,
      stake: stake.publicKey,
      memberSigner,
      registrarSigner,
      tokenProgram: TOKEN_PROGRAM_ID,
    }).signers([beneficiary]).rpc();

    const [availableAccount, stakeAccount, sptAccount] = await Promise.all(
      [available, stake, spt].map((v) =>
        getAccount(
          connection,
          v.publicKey,
        )
      ),
    );

    expect(availableAccount.amount).to.eql(BigInt(100));
    expect(stakeAccount.amount).to.eql(BigInt(20));
    expect(sptAccount.amount).to.eql(BigInt(10));
  });

  const claimReward = async (
    vendor: PublicKey,
    vendorVault: PublicKey,
    to: PublicKey,
  ) => {
    await registry.methods.claimReward().accounts({
      to,
      registrar: registrar.publicKey,
      member: member.publicKey,
      beneficiary: beneficiary.publicKey,
      spt: spt.publicKey,
      vendor,
      vendorVault: vendorVault,
      registrarSigner,
      tokenProgram: TOKEN_PROGRAM_ID,
    }).signers([beneficiary]).rpc();
  };

  const vendor = new Keypair();

  it("drops reward", async () => {
    const amount = 200;
    const expiry = new BN(Date.now() / 1000 + 9);

    await registry.methods.dropReward(
      new BN(amount),
      0.0,
      expiry,
      god.publicKey,
    ).accounts({
      registrar: registrar.publicKey,
      rewardQueue: rewardQueue.publicKey,
      poolMint,
      vendor: vendor.publicKey,
      vendorVault: vendorVault.publicKey,
      depositor: godDepositor,
      depositorAuthority: god.publicKey,
      tokenProgram: TOKEN_PROGRAM_ID,
    }).signers([vendor, god]).preInstructions([
      await registry.account.rewardVendor.createInstruction(vendor),
    ]).rpc();
  });

  it("claims reward", async () => {
    const amount_before = (await getAccount(
      connection,
      beneficiaryDepositor,
    )).amount;

    await claimReward(
      vendor.publicKey,
      vendorVault.publicKey,
      beneficiaryDepositor,
    );

    const amount_after = (await getAccount(
      connection,
      beneficiaryDepositor,
    )).amount;

    expect(amount_after - amount_before).to.eq(BigInt(200));
  });

  const pendingWithdrawal = new Keypair();

  it("starts unstake", async () => {
    const amount = 10;

    await registry.methods.startUnstake(new BN(amount)).accounts({
      registrar: registrar.publicKey,
      rewardQueue: rewardQueue.publicKey,
      poolMint,
      pendingWithdrawal: pendingWithdrawal.publicKey,
      member: member.publicKey,
      beneficiary: beneficiary.publicKey,
      spt: spt.publicKey,
      stake: stake.publicKey,
      pending: pending.publicKey,
      memberSigner,
      tokenProgram: TOKEN_PROGRAM_ID,
    }).signers([beneficiary, pendingWithdrawal]).preInstructions([
      await registry.account.pendingWithdrawal.createInstruction(
        pendingWithdrawal,
      ),
    ]).rpc();

    const [sptAccount, stakeAccount, pendingAccount] = await Promise.all(
      [spt, stake, pending].map((v) =>
        getAccount(
          connection,
          v.publicKey,
        )
      ),
    );

    expect(stakeAccount.amount).to.eql(BigInt(0));
    expect(sptAccount.amount).to.eql(BigInt(0));
    expect(pendingAccount.amount).to.eql(BigInt(20));
  });

  const endUnstake = async () => {
    await registry.methods.endUnstake().accounts({
      registrar: registrar.publicKey,
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

    expect(amount_after - amount_before).to.eq(BigInt(20));
  });

  it("withdraws", async () => {
    const amount = 100;

    const amount_before = (await getAccount(
      connection,
      beneficiaryDepositor,
    )).amount;

    await registry.methods.withdraw(new BN(amount)).accounts({
      registrar: registrar.publicKey,
      member: member.publicKey,
      beneficiary: beneficiary.publicKey,
      available: available.publicKey,
      memberSigner,
      depositor: beneficiaryDepositor,
      tokenProgram: TOKEN_PROGRAM_ID,
    }).signers([beneficiary]).rpc();

    const amount_after = (await getAccount(
      connection,
      beneficiaryDepositor,
    )).amount;

    expect(amount_after - amount_before).to.eq(BigInt(amount));
  });
});
