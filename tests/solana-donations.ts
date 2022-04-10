import * as anchor from "@project-serum/anchor";
import { BN, Program } from "@project-serum/anchor";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import {
  createAccount,
  createMint,
  getAccount,
  mintTo,
  TOKEN_PROGRAM_ID,
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
  const beneficiary = new Keypair();

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
  const vendorVault = new Keypair();

  it("initializes registry", async () => {
    await createAccount(connection, payer, mint, registrarSigner, vendorVault);
    await mintTo(
      connection,
      payer,
      mint,
      vendorVault.publicKey,
      mintAuthority,
      1000000,
    );

    await registry.methods.initialize(
      registrarSignerNonce,
      mint,
      registryAuthority.publicKey,
      new BN(2),
      new BN(2),
      new BN(170),
    ).accounts({
      registrar: registrar.publicKey,
      poolMint,
      vendorVault: vendorVault.publicKey,
      registrarSigner,
    }).signers([registrar]).preInstructions(
      [await registry.account.registrar.createInstruction(registrar)],
    ).rpc();
  });

  const member = new Keypair();
  let memberSigner: PublicKey;
  let memberSignerNonce: number;
  const available = new Keypair();
  const stake = new Keypair();
  const pending = new Keypair();

  it("creates member", async () => {
    [memberSigner, memberSignerNonce] = await PublicKey.findProgramAddress(
      [registrar.publicKey.toBuffer(), member.publicKey.toBuffer()],
      registry.programId,
    );

    await Promise.all([
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
      poolMint,
      member: member.publicKey,
      beneficiary: beneficiary.publicKey,
      available: available.publicKey,
      stake: stake.publicKey,
      memberSigner,
      registrarSigner,
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

    await registry.methods.claimReward().accounts({
      to: beneficiaryDepositor,
      registrar: registrar.publicKey,
      member: member.publicKey,
      beneficiary: beneficiary.publicKey,
      vendorVault: vendorVault.publicKey,
      registrarSigner,
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

    await registry.methods.startUnstake(new BN(amount)).accounts({
      registrar: registrar.publicKey,
      poolMint,
      pendingWithdrawal: pendingWithdrawal.publicKey,
      member: member.publicKey,
      beneficiary: beneficiary.publicKey,
      stake: stake.publicKey,
      pending: pending.publicKey,
      memberSigner,
      tokenProgram: TOKEN_PROGRAM_ID,
    }).signers([beneficiary, pendingWithdrawal]).preInstructions([
      await registry.account.pendingWithdrawal.createInstruction(
        pendingWithdrawal,
      ),
    ]).rpc();

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

    expect(amount_after - amount_before).to.eq(BigInt(10));
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
