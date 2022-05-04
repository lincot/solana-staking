import { BN } from "@project-serum/anchor";
import { Keypair, SystemProgram } from "@solana/web3.js";
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
  ctx.stakingId = (
    await ctx.program.account.factory.fetch(ctx.factory)
  ).stakingsCount;

  await ctx.program.methods
    .createStaking(ctx.stakeMint, withdrawalTimelock, rewardType)
    .accounts({
      factory: ctx.factory,
      staking: await ctx.staking(),
      configHistory: await ctx.configHistory(),
      stakesHistory: await ctx.stakesHistory(),
      rewardMint: ctx.rewardMint,
      authority: ctx.stakingAuthority.publicKey,
      systemProgram: SystemProgram.programId,
    })
    .signers([ctx.stakingAuthority])
    .rpc();

  await mintTo(
    ctx,
    await ctx.rewardATA(await ctx.staking()),
    ctx.mintAuthority,
    1_000_000
  );
}

export async function changeConfig(
  ctx: Context,
  rewardType: any
): Promise<void> {
  await ctx.program.methods
    .changeConfig(rewardType)
    .accounts({
      staking: await ctx.staking(),
      configHistory: await ctx.configHistory(),
      stakesHistory: await ctx.stakesHistory(),
      authority: ctx.stakingAuthority.publicKey,
      systemProgram: SystemProgram.programId,
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
      staking: await ctx.staking(),
      beneficiary: beneficiary.publicKey,
      member: await ctx.member(beneficiary.publicKey),
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
  await mintTo(
    ctx,
    await ctx.stakeATA(beneficiary.publicKey),
    ctx.mintAuthority,
    Number(amount)
  );

  await ctx.program.methods
    .deposit(new BN(amount))
    .accounts({
      staking: await ctx.staking(),
      beneficiary: beneficiary.publicKey,
      member: await ctx.member(beneficiary.publicKey),
      from: await ctx.stakeATA(beneficiary.publicKey),
      memberVault: await ctx.stakeATA(await ctx.member(beneficiary.publicKey)),
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
      staking: await ctx.staking(),
      configHistory: await ctx.configHistory(),
      stakesHistory: await ctx.stakesHistory(),
      beneficiary: beneficiary.publicKey,
      member: await ctx.member(beneficiary.publicKey),
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
      factoryVault: ctx.factoryVault,
      staking: await ctx.staking(),
      stakingVault: await ctx.rewardATA(await ctx.staking()),
      configHistory: await ctx.configHistory(),
      stakesHistory: await ctx.stakesHistory(),
      beneficiary: beneficiary.publicKey,
      member: await ctx.member(beneficiary.publicKey),
      to: await ctx.rewardATA(beneficiary.publicKey),
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
      staking: await ctx.staking(),
      configHistory: await ctx.configHistory(),
      stakesHistory: await ctx.stakesHistory(),
      beneficiary: beneficiary.publicKey,
      member: await ctx.member(beneficiary.publicKey),
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
      staking: await ctx.staking(),
      beneficiary: beneficiary.publicKey,
      member: await ctx.member(beneficiary.publicKey),
    })
    .signers([beneficiary])
    .rpc();
}

export async function withdraw(
  ctx: Context,
  beneficiary: Keypair,
  amount: number | BN
): Promise<void> {
  await ctx.program.methods
    .withdraw(new BN(amount))
    .accounts({
      staking: await ctx.staking(),
      beneficiary: beneficiary.publicKey,
      member: await ctx.member(beneficiary.publicKey),
      memberVault: await ctx.stakeATA(await ctx.member(beneficiary.publicKey)),
      to: await ctx.stakeATA(beneficiary.publicKey),
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .signers([beneficiary])
    .rpc();
}
