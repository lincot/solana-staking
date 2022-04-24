import { BN } from "@project-serum/anchor";
import { Keypair, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { Context } from "./ctx";
import { findATA, mintTo } from "./token";

export async function initialize(ctx: Context): Promise<void> {
  await ctx.program.methods
    .initialize()
    .accounts({
      factory: ctx.factory,
      authority: ctx.factoryAuthority.publicKey,
      systemProgram: SystemProgram.programId,
    })
    .signers([ctx.factoryAuthority])
    .rpc();
}

export async function createStaking(
  ctx: Context,
  withdrawalTimelock: number,
  rewardType: any
): Promise<void> {
  await ctx.program.methods
    .createStaking(ctx.mint, withdrawalTimelock, rewardType)
    .accounts({
      factory: ctx.factory,
      staking: ctx.staking,
      rewardVault: ctx.rewardVault,
      rewardMint: ctx.mint,
      authority: ctx.stakingAuthority.publicKey,
      rent: SYSVAR_RENT_PUBKEY,
      systemProgram: SystemProgram.programId,
    })
    .signers([ctx.stakingAuthority])
    .rpc();

  await mintTo(ctx, ctx.rewardVault, ctx.mintAuthority, 1_000_000);
}

export async function changeConfig(
  ctx: Context,
  rewardType: any
): Promise<void> {
  await ctx.program.methods
    .changeConfig(rewardType)
    .accounts({
      staking: ctx.staking,
      authority: ctx.stakingAuthority.publicKey,
    })
    .signers([ctx.stakingAuthority])
    .rpc();
}

export async function registerMember(
  ctx: Context,
  beneficiary: Keypair
): Promise<void> {
  await ctx.program.methods
    .registerMember()
    .accounts({
      staking: ctx.staking,
      stakeMint: ctx.mint,
      member: await ctx.member(beneficiary.publicKey),
      pendingWithdrawal: await ctx.pendingWithdrawal(beneficiary.publicKey),
      beneficiary: beneficiary.publicKey,
      available: await ctx.available(beneficiary.publicKey),
      stake: await ctx.stake(beneficiary.publicKey),
      pending: await ctx.pending(beneficiary.publicKey),
      rent: SYSVAR_RENT_PUBKEY,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    })
    .signers([beneficiary])
    .rpc();
}

export async function deposit(
  ctx: Context,
  beneficiary: Keypair,
  amount: number | BN
): Promise<void> {
  await ctx.program.methods
    .deposit(new BN(amount))
    .accounts({
      staking: ctx.staking,
      member: await ctx.member(beneficiary.publicKey),
      beneficiary: beneficiary.publicKey,
      available: await ctx.available(beneficiary.publicKey),
      depositor: await findATA(ctx, beneficiary.publicKey, ctx.mint),
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .signers([beneficiary])
    .rpc();
}

export async function stake(
  ctx: Context,
  beneficiary: Keypair,
  amount: number | BN
): Promise<void> {
  await ctx.program.methods
    .stake(new BN(amount))
    .accounts({
      staking: ctx.staking,
      member: await ctx.member(beneficiary.publicKey),
      beneficiary: beneficiary.publicKey,
      available: await ctx.available(beneficiary.publicKey),
      stake: await ctx.stake(beneficiary.publicKey),
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .signers([beneficiary])
    .rpc();
}

export async function claimReward(
  ctx: Context,
  beneficiary: Keypair
): Promise<void> {
  await ctx.program.methods
    .claimReward()
    .accounts({
      factory: ctx.factory,
      staking: ctx.staking,
      member: await ctx.member(beneficiary.publicKey),
      beneficiary: beneficiary.publicKey,
      stake: await ctx.stake(beneficiary.publicKey),
      rewardVault: ctx.rewardVault,
      destination: await findATA(ctx, beneficiary.publicKey, ctx.mint),
      factoryVault: ctx.factoryVault,
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .signers([beneficiary])
    .rpc();
}

export async function startUnstake(
  ctx: Context,
  beneficiary: Keypair,
  amount: number | BN
): Promise<void> {
  await ctx.program.methods
    .startUnstake(new BN(amount))
    .accounts({
      staking: ctx.staking,
      pendingWithdrawal: await ctx.pendingWithdrawal(beneficiary.publicKey),
      member: await ctx.member(beneficiary.publicKey),
      beneficiary: beneficiary.publicKey,
      stake: await ctx.stake(beneficiary.publicKey),
      pending: await ctx.pending(beneficiary.publicKey),
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    })
    .signers([beneficiary])
    .rpc();
}

export async function endUnstake(
  ctx: Context,
  beneficiary: Keypair
): Promise<void> {
  await ctx.program.methods
    .endUnstake()
    .accounts({
      staking: ctx.staking,
      member: await ctx.member(beneficiary.publicKey),
      beneficiary: beneficiary.publicKey,
      pendingWithdrawal: await ctx.pendingWithdrawal(beneficiary.publicKey),
      available: await ctx.available(beneficiary.publicKey),
      pending: await ctx.pending(beneficiary.publicKey),
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .signers([beneficiary])
    .rpc();
}

export async function withdraw(
  ctx: Context,
  beneficiary: Keypair
): Promise<void> {
  await ctx.program.methods
    .withdraw(new BN(100))
    .accounts({
      staking: ctx.staking,
      member: await ctx.member(beneficiary.publicKey),
      beneficiary: beneficiary.publicKey,
      available: await ctx.available(beneficiary.publicKey),
      destination: await findATA(ctx, beneficiary.publicKey, ctx.mint),
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .signers([beneficiary])
    .rpc();
}
