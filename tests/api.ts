import { BN } from "@project-serum/anchor";
import { Keypair, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { Context } from "./ctx";
import { mintTo } from "./token";

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

  await ctx.program.methods
    .createStaking(ctx.stakeMint, withdrawalTimelock, rewardType)
    .accounts({
      factory: ctx.factory,
      staking,
      rewardVault,
      configHistory: await ctx.configHistory(staking),
      stakesHistory: await ctx.stakesHistory(staking),
      rewardMint: ctx.rewardMint,
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

  await ctx.program.methods
    .changeConfig(rewardType)
    .accounts({
      staking,
      configHistory: await ctx.configHistory(staking),
      stakesHistory: await ctx.stakesHistory(staking),
      authority: ctx.stakingAuthority.publicKey,
      systemProgram: SystemProgram.programId,
    })
    .signers([ctx.stakingAuthority])
    .rpc();
}

export async function registerMember(
  ctx: Context,
  stakingId: number | BN,
  beneficiary: Keypair
): Promise<void> {
  await ctx.program.methods
    .registerMember()
    .accounts({
      staking: await ctx.staking(stakingId),
      beneficiary: beneficiary.publicKey,
      member: await ctx.member(beneficiary.publicKey, stakingId),
      pendingWithdrawal: await ctx.pendingWithdrawal(
        beneficiary.publicKey,
        stakingId
      ),
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
  const member = await ctx.member(beneficiary.publicKey, stakingId);

  await ctx.program.methods
    .deposit(new BN(amount))
    .accounts({
      staking: await ctx.staking(stakingId),
      beneficiary: beneficiary.publicKey,
      member,
      from: await ctx.stakeATA(beneficiary.publicKey),
      memberVault: await ctx.stakeATA(member),
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

  await ctx.program.methods
    .stake(new BN(amount))
    .accounts({
      staking,
      configHistory: await ctx.configHistory(staking),
      stakesHistory: await ctx.stakesHistory(staking),
      beneficiary: beneficiary.publicKey,
      member: await ctx.member(beneficiary.publicKey, stakingId),
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

  await ctx.program.methods
    .claimReward()
    .accounts({
      factory: ctx.factory,
      factoryVault: ctx.factoryVault,
      staking,
      configHistory: await ctx.configHistory(staking),
      stakesHistory: await ctx.stakesHistory(staking),
      beneficiary: beneficiary.publicKey,
      member: await ctx.member(beneficiary.publicKey, stakingId),
      rewardVault: await ctx.rewardVault(staking),
      to: await ctx.rewardATA(beneficiary.publicKey),
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

  await ctx.program.methods
    .startUnstake(new BN(amount))
    .accounts({
      staking,
      configHistory: await ctx.configHistory(staking),
      stakesHistory: await ctx.stakesHistory(staking),
      beneficiary: beneficiary.publicKey,
      member: await ctx.member(beneficiary.publicKey, stakingId),
      pendingWithdrawal: await ctx.pendingWithdrawal(
        beneficiary.publicKey,
        stakingId
      ),
    })
    .signers([beneficiary])
    .rpc();
}

export async function endUnstake(
  ctx: Context,
  stakingId: number | BN,
  beneficiary: Keypair
): Promise<void> {
  await ctx.program.methods
    .endUnstake()
    .accounts({
      staking: await ctx.staking(stakingId),
      beneficiary: beneficiary.publicKey,
      member: await ctx.member(beneficiary.publicKey, stakingId),
      pendingWithdrawal: await ctx.pendingWithdrawal(
        beneficiary.publicKey,
        stakingId
      ),
    })
    .signers([beneficiary])
    .rpc();
}

export async function withdraw(
  ctx: Context,
  stakingId: number | BN,
  beneficiary: Keypair
): Promise<void> {
  const member = await ctx.member(beneficiary.publicKey, stakingId);

  await ctx.program.methods
    .withdraw(new BN(100))
    .accounts({
      staking: await ctx.staking(stakingId),
      beneficiary: beneficiary.publicKey,
      member,
      memberVault: await ctx.stakeATA(member),
      to: await ctx.stakeATA(beneficiary.publicKey),
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .signers([beneficiary])
    .rpc();
}
