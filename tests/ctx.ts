import * as anchor from "@project-serum/anchor";
import { BN, Program } from "@project-serum/anchor";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import { StakingFactory } from "../target/types/staking_factory";
import { createMint, findATA, mintTo, TokenAccount } from "./token";
import { airdrop, findPDA } from "./utils";

export class Context {
  connection: Connection;
  program: Program<StakingFactory>;
  payer: Keypair;

  mintAuthority: Keypair;
  mint: PublicKey;

  factoryAuthority: Keypair;
  factory: PublicKey;
  factoryVault: TokenAccount;

  stakingAuthority: Keypair;
  staking: PublicKey;
  rewardVault: TokenAccount;

  beneficiary: Keypair;

  constructor() {
    this.connection = new Connection("http://localhost:8899", "recent");
    this.program = anchor.workspace.StakingFactory;
    this.payer = new Keypair();

    this.mintAuthority = new Keypair();
    this.factoryAuthority = new Keypair();
    this.stakingAuthority = new Keypair();
    this.beneficiary = new Keypair();
  }

  async setup() {
    await airdrop(this, [
      this.mintAuthority.publicKey,
      this.factoryAuthority.publicKey,
      this.stakingAuthority.publicKey,
      this.beneficiary.publicKey,
    ]);

    this.mint = await createMint(this, this.mintAuthority, 2);

    this.factory = await findPDA(this, [Buffer.from("factory")]);

    this.factoryVault = new TokenAccount(
      await findATA(this, this.factoryAuthority.publicKey, this.mint),
      this.mint
    );

    this.staking = await findPDA(this, [
      Buffer.from("staking"),
      new BN(0).toArrayLike(Buffer, "le", 2),
    ]);
    this.rewardVault = new TokenAccount(
      await findPDA(this, [
        Buffer.from("reward_vault"),
        this.staking.toBuffer(),
      ]),
      this.mint
    );

    await mintTo(
      this,
      await findATA(this, this.beneficiary.publicKey, this.mint),
      this.mintAuthority,
      100
    );
  }

  async member(user: PublicKey): Promise<PublicKey> {
    return await findPDA(this, [
      Buffer.from("member"),
      new BN(0).toArrayLike(Buffer, "le", 2),
      user.toBuffer(),
    ]);
  }

  async pendingWithdrawal(user: PublicKey): Promise<PublicKey> {
    const member = await this.member(user);
    return await findPDA(this, [
      Buffer.from("pending_withdrawal"),
      member.toBuffer(),
    ]);
  }

  async available(user: PublicKey): Promise<TokenAccount> {
    const member = await this.member(user);
    const address = await findPDA(this, [
      Buffer.from("available"),
      member.toBuffer(),
    ]);
    return new TokenAccount(address, this.mint);
  }

  async stake(user: PublicKey): Promise<TokenAccount> {
    const member = await this.member(user);
    const address = await findPDA(this, [
      Buffer.from("stake"),
      member.toBuffer(),
    ]);
    return new TokenAccount(address, this.mint);
  }

  async pending(user: PublicKey): Promise<TokenAccount> {
    const member = await this.member(user);
    const address = await findPDA(this, [
      Buffer.from("pending"),
      member.toBuffer(),
    ]);
    return new TokenAccount(address, this.mint);
  }

  async findATA(owner: PublicKey): Promise<TokenAccount> {
    return await findATA(this, owner, this.mint);
  }
}
