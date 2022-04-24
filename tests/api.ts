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
  const stakingId = (await ctx.program.account.factory.fetch(ctx.factory))
    .stakingsCount;
  const staking = await ctx.staking(stakingId);
  const rewardVault = await ctx.rewardVault(staking);
  const configHistory = await ctx.configHistory(staking);

  await ctx.program.methods
    .createStaking(ctx.mint, withdrawalTimelock, rewardType)
    .accounts({
      factory: ctx.factory,
      staking,
      rewardVault,
      configHistory,
      rewardMint: ctx.mint,
      authority: ctx.stakingAuthority.publicKey,
      rent: SYSVAR_RENT_PUBKEY,
      systemProgram: SystemProgram.programId,
    })
    .signers([ctx.stakingAuthority])
    .rpc();

  await mintTo(ctx, rewardVault, ctx.mintAuthority, 1_000_000);
}

export async function changeConfig(
  ctx: Context,
  stakingId: number | BN,
  rewardType: any
): Promise<void> {
  const staking = await ctx.staking(stakingId);
  const configHistory = await ctx.configHistory(staking);

  await ctx.program.methods
    .changeConfig(rewardType)
    .accounts({
      staking,
      configHistory,
      authority: ctx.stakingAuthority.publicKey,
    })
    .signers([ctx.stakingAuthority])
    .rpc();
}

export async function registerMember(
  ctx: Context,
  stakingId: number | BN,
  beneficiary: Keypair
): Promise<void> {
  const staking = await ctx.staking(stakingId);

  await ctx.program.methods
    .registerMember()
    .accounts({
      staking,
      stakeMint: ctx.mint,
      member: await ctx.member(beneficiary.publicKey, stakingId),
      pendingWithdrawal: await ctx.pendingWithdrawal(
        beneficiary.publicKey,
        stakingId
      ),
      beneficiary: beneficiary.publicKey,
      available: await ctx.available(beneficiary.publicKey, stakingId),
      stake: await ctx.stake(beneficiary.publicKey, stakingId),
      pending: await ctx.pending(beneficiary.publicKey, stakingId),
      rent: SYSVAR_RENT_PUBKEY,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    })
    .signers([beneficiary])
    .rpc();
}

export async function deposit(
  ctx: Context,
  stakingId: number | BN,
  beneficiary: Keypair,
  amount: number | BN
): Promise<void> {
  const staking = await ctx.staking(stakingId);

  await ctx.program.methods
    .deposit(new BN(amount))
    .accounts({
      staking,
      member: await ctx.member(beneficiary.publicKey, stakingId),
      beneficiary: beneficiary.publicKey,
      available: await ctx.available(beneficiary.publicKey, stakingId),
      depositor: await findATA(ctx, beneficiary.publicKey, ctx.mint),
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .signers([beneficiary])
    .rpc();
}

export async function stake(
  ctx: Context,
  stakingId: number | BN,
  beneficiary: Keypair,
  amount: number | BN
): Promise<void> {
  const staking = await ctx.staking(stakingId);
  const configHistory = await ctx.configHistory(staking);

  await ctx.program.methods
    .stake(new BN(amount))
    .accounts({
      staking,
      configHistory,
      member: await ctx.member(beneficiary.publicKey, stakingId),
      beneficiary: beneficiary.publicKey,
      available: await ctx.available(beneficiary.publicKey, stakingId),
      stake: await ctx.stake(beneficiary.publicKey, stakingId),
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .signers([beneficiary])
    .rpc();
}

export async function claimReward(
  ctx: Context,
  stakingId: number | BN,
  beneficiary: Keypair
): Promise<void> {
  const staking = await ctx.staking(stakingId);
  const rewardVault = await ctx.rewardVault(staking);
  const configHistory = await ctx.configHistory(staking);

  await ctx.program.methods
    .claimReward()
    .accounts({
      factory: ctx.factory,
      staking,
      configHistory,
      member: await ctx.member(beneficiary.publicKey, stakingId),
      beneficiary: beneficiary.publicKey,
      stake: await ctx.stake(beneficiary.publicKey, stakingId),
      rewardVault,
      destination: await findATA(ctx, beneficiary.publicKey, ctx.mint),
      factoryVault: ctx.factoryVault,
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .signers([beneficiary])
    .rpc();
}

export async function startUnstake(
  ctx: Context,
  stakingId: number | BN,
  beneficiary: Keypair,
  amount: number | BN
): Promise<void> {
  const staking = await ctx.staking(stakingId);
  const configHistory = await ctx.configHistory(staking);

  await ctx.program.methods
    .startUnstake(new BN(amount))
    .accounts({
      staking,
      configHistory,
      pendingWithdrawal: await ctx.pendingWithdrawal(
        beneficiary.publicKey,
        stakingId
      ),
      member: await ctx.member(beneficiary.publicKey, stakingId),
      beneficiary: beneficiary.publicKey,
      stake: await ctx.stake(beneficiary.publicKey, stakingId),
      pending: await ctx.pending(beneficiary.publicKey, stakingId),
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    })
    .signers([beneficiary])
    .rpc();
}

export async function endUnstake(
  ctx: Context,
  stakingId: number | BN,
  beneficiary: Keypair
): Promise<void> {
  const staking = await ctx.staking(stakingId);

  await ctx.program.methods
    .endUnstake()
    .accounts({
      staking,
      member: await ctx.member(beneficiary.publicKey, stakingId),
      beneficiary: beneficiary.publicKey,
      pendingWithdrawal: await ctx.pendingWithdrawal(
        beneficiary.publicKey,
        stakingId
      ),
      available: await ctx.available(beneficiary.publicKey, stakingId),
      pending: await ctx.pending(beneficiary.publicKey, stakingId),
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .signers([beneficiary])
    .rpc();
}

export async function withdraw(
  ctx: Context,
  stakingId: number | BN,
  beneficiary: Keypair
): Promise<void> {
  const staking = await ctx.staking(stakingId);

  await ctx.program.methods
    .withdraw(new BN(100))
    .accounts({
      staking,
      member: await ctx.member(beneficiary.publicKey, stakingId),
      beneficiary: beneficiary.publicKey,
      available: await ctx.available(beneficiary.publicKey, stakingId),
      destination: await findATA(ctx, beneficiary.publicKey, ctx.mint),
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .signers([beneficiary])
    .rpc();
}