import * as anchor from "@project-serum/anchor";
import { BN, Program } from "@project-serum/anchor";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import { StakingFactory } from "../target/types/staking_factory";
import { createMint, findATA, TokenAccount } from "./token";
import { airdrop, findPDA } from "./utils";

export class Context {
  connection: Connection;
  program: Program<StakingFactory>;
  payer: Keypair;

  mintAuthority: Keypair;
  stakeMint: PublicKey;
  rewardMint: PublicKey;

  factoryAuthority: Keypair;
  factory: PublicKey;
  factoryVault: TokenAccount;

  stakingAuthority: Keypair;

  user1: Keypair;
  user2: Keypair;

  constructor() {
    this.connection = new Connection("http://localhost:8899", "recent");
    this.program = anchor.workspace.StakingFactory;
    this.payer = new Keypair();

    this.mintAuthority = new Keypair();
    this.factoryAuthority = new Keypair();
    this.stakingAuthority = new Keypair();
    this.user1 = new Keypair();
    this.user2 = new Keypair();
  }

  async setup() {
    await airdrop(this, [
      this.mintAuthority.publicKey,
      this.factoryAuthority.publicKey,
      this.stakingAuthority.publicKey,
      this.user1.publicKey,
      this.user2.publicKey,
    ]);

    this.stakeMint = await createMint(this, this.mintAuthority, 2);
    this.rewardMint = await createMint(this, this.mintAuthority, 6);

    this.factory = await findPDA(this, [Buffer.from("factory")]);

    this.factoryVault = await this.rewardATA(this.factoryAuthority.publicKey);
  }

  async staking(stakingId: number | BN): Promise<PublicKey> {
    return await findPDA(this, [
      Buffer.from("staking"),
      new BN(stakingId).toArrayLike(Buffer, "le", 2),
    ]);
  }

  async rewardVault(staking: PublicKey): Promise<TokenAccount> {
    return new TokenAccount(
      await findPDA(this, [Buffer.from("reward_vault"), staking.toBuffer()]),
      this.rewardMint
    );
  }

  async configHistory(staking: PublicKey): Promise<PublicKey> {
    return await findPDA(this, [
      Buffer.from("config_history"),
      staking.toBuffer(),
    ]);
  }

  async stakesHistory(staking: PublicKey): Promise<PublicKey> {
    return await findPDA(this, [
      Buffer.from("stakes_history"),
      staking.toBuffer(),
    ]);
  }

  async member(user: PublicKey, stakingId: number | BN): Promise<PublicKey> {
    return await findPDA(this, [
      Buffer.from("member"),
      new BN(stakingId).toArrayLike(Buffer, "le", 2),
      user.toBuffer(),
    ]);
  }

  async pendingWithdrawal(
    user: PublicKey,
    stakingId: number | BN
  ): Promise<PublicKey> {
    const member = await this.member(user, stakingId);
    return await findPDA(this, [
      Buffer.from("pending_withdrawal"),
      member.toBuffer(),
    ]);
  }

  async available(
    user: PublicKey,
    stakingId: number | BN
  ): Promise<TokenAccount> {
    const member = await this.member(user, stakingId);
    const address = await findPDA(this, [
      Buffer.from("available"),
      member.toBuffer(),
    ]);
    return new TokenAccount(address, this.stakeMint);
  }

  async stake(user: PublicKey, stakingId: number | BN): Promise<TokenAccount> {
    const member = await this.member(user, stakingId);
    const address = await findPDA(this, [
      Buffer.from("stake"),
      member.toBuffer(),
    ]);
    return new TokenAccount(address, this.stakeMint);
  }

  async pending(
    user: PublicKey,
    stakingId: number | BN
  ): Promise<TokenAccount> {
    const member = await this.member(user, stakingId);
    const address = await findPDA(this, [
      Buffer.from("pending"),
      member.toBuffer(),
    ]);
    return new TokenAccount(address, this.stakeMint);
  }

  async stakeATA(owner: PublicKey): Promise<TokenAccount> {
    return await findATA(this, owner, this.stakeMint);
  }

  async rewardATA(owner: PublicKey): Promise<TokenAccount> {
    return await findATA(this, owner, this.rewardMint);
  }
}
