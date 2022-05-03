import { BN } from "@project-serum/anchor";
import { expect } from "chai";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { sleep } from "./utils";
import { Context } from "./ctx";
import {
  changeConfig,
  claimReward,
  createStaking,
  deposit,
  endUnstake,
  initialize,
  registerMember,
  stake,
  startUnstake,
  withdraw,
} from "./api";
import { burnAll, mintTo } from "./token";

chai.use(chaiAsPromised);

const ctx = new Context();

describe("setup", () => {
  it("setups", async () => {
    await ctx.setup();
  });
});

describe("factory", () => {
  it("initializes", async () => {
    await initialize(ctx);
  });
});

describe("interest rate", () => {
  it("creates staking", async () => {
    await createStaking(ctx, 2, {
      interestRate: { num: new BN(1337), denom: new BN(100) },
    });
  });

  it("changes config", async () => {
    await changeConfig(ctx, 0, {
      interestRate: { num: new BN(10), denom: new BN(100) },
    });
  });

  it("registers", async () => {
    await registerMember(ctx, 0, ctx.user1);
    await mintTo(
      ctx,
      await ctx.stakeATA(ctx.user1.publicKey),
      ctx.mintAuthority,
      100
    );
  });

  it("deposits", async () => {
    await deposit(ctx, 0, ctx.user1, 100);

    const memberAccount = await ctx.program.account.member.fetch(
      await ctx.member(ctx.user1.publicKey, 0)
    );

    expect(memberAccount.availableAmount.toNumber()).to.eq(100);
  });

  it("stakes", async () => {
    await stake(ctx, 0, ctx.user1, 100);

    const memberAccount = await ctx.program.account.member.fetch(
      await ctx.member(ctx.user1.publicKey, 0)
    );

    expect(memberAccount.availableAmount.toNumber()).to.eq(0);
    expect(memberAccount.stakeAmount.toNumber()).to.eq(100);
  });

  it("waits", async () => {
    await sleep(3000);
  });

  it("claims", async () => {
    await claimReward(ctx, 0, ctx.user1);

    expect(
      await (await ctx.rewardATA(ctx.user1.publicKey)).amount(ctx)
    ).to.be.oneOf([39, 49, 59]);
    expect(await ctx.factoryVault.amount(ctx)).to.eq(1);

    await burnAll(ctx, await ctx.rewardATA(ctx.user1.publicKey), ctx.user1);
    await burnAll(ctx, ctx.factoryVault, ctx.factoryAuthority);
  });

  it("starts unstake", async () => {
    await startUnstake(ctx, 0, ctx.user1, 100);

    const memberAccount = await ctx.program.account.member.fetch(
      await ctx.member(ctx.user1.publicKey, 0)
    );

    expect(memberAccount.stakeAmount.toNumber()).to.eq(0);
    expect(memberAccount.pendingAmount.toNumber()).to.eq(100);
  });

  it("fails to end unstake before timelock", async () => {
    await expect(endUnstake(ctx, 0, ctx.user1)).to.be.rejected;
  });

  it("waits for unstake timelock to end", async () => {
    await sleep(2000);
  });

  it("ends unstake", async () => {
    await endUnstake(ctx, 0, ctx.user1);

    const memberAccount = await ctx.program.account.member.fetch(
      await ctx.member(ctx.user1.publicKey, 0)
    );

    expect(memberAccount.availableAmount.toNumber()).to.eq(100);
  });

  it("withdraws", async () => {
    await withdraw(ctx, 0, ctx.user1);

    expect(await (await ctx.stakeATA(ctx.user1.publicKey)).amount(ctx)).to.eq(
      100
    );
  });
});

describe("proportional", () => {
  it("creates staking", async () => {
    await createStaking(ctx, 2, {
      proportional: { totalAmount: new BN(100), rewardPeriod: 3 },
    });
  });

  it("registers", async () => {
    await registerMember(ctx, 1, ctx.user1);
    await registerMember(ctx, 1, ctx.user2);
    await mintTo(
      ctx,
      await ctx.stakeATA(ctx.user2.publicKey),
      ctx.mintAuthority,
      100
    );
  });

  it("deposits", async () => {
    await deposit(ctx, 1, ctx.user1, 100);
    await deposit(ctx, 1, ctx.user2, 100);

    const memberAccount = await ctx.program.account.member.fetch(
      await ctx.member(ctx.user1.publicKey, 1)
    );

    expect(memberAccount.availableAmount.toNumber()).to.eq(100);
  });

  it("stakes", async () => {
    await stake(ctx, 1, ctx.user1, 100);
    await stake(ctx, 1, ctx.user2, 100);

    const memberAccount = await ctx.program.account.member.fetch(
      await ctx.member(ctx.user1.publicKey, 1)
    );

    expect(memberAccount.availableAmount.toNumber()).to.eq(0);
    expect(memberAccount.stakeAmount.toNumber()).to.eq(100);
  });

  it("waits", async () => {
    await sleep(2000);
  });

  it("claims", async () => {
    await claimReward(ctx, 1, ctx.user1);
    await claimReward(ctx, 1, ctx.user2);

    expect(await (await ctx.rewardATA(ctx.user1.publicKey)).amount(ctx)).to.eq(
      49
    );
    expect(await (await ctx.rewardATA(ctx.user2.publicKey)).amount(ctx)).to.eq(
      49
    );
    expect(await ctx.factoryVault.amount(ctx)).to.eq(2);

    await burnAll(ctx, await ctx.rewardATA(ctx.user1.publicKey), ctx.user1);
    await burnAll(ctx, await ctx.rewardATA(ctx.user2.publicKey), ctx.user2);
    await burnAll(ctx, ctx.factoryVault, ctx.factoryAuthority);
  });
});

describe("fixed", () => {
  it("creates staking", async () => {
    await createStaking(ctx, 2, {
      fixed: {
        requiredAmount: new BN(100),
        requiredPeriod: 3,
        rewardAmount: new BN(100),
      },
    });
  });

  it("registers", async () => {
    await registerMember(ctx, 2, ctx.user1);
    await mintTo(
      ctx,
      await ctx.stakeATA(ctx.user1.publicKey),
      ctx.mintAuthority,
      100
    );
  });

  it("deposits", async () => {
    await deposit(ctx, 2, ctx.user1, 100);

    const memberAccount = await ctx.program.account.member.fetch(
      await ctx.member(ctx.user1.publicKey, 2)
    );

    expect(memberAccount.availableAmount.toNumber()).to.eq(100);
  });

  it("stakes", async () => {
    await stake(ctx, 2, ctx.user1, 100);

    const memberAccount = await ctx.program.account.member.fetch(
      await ctx.member(ctx.user1.publicKey, 2)
    );

    expect(memberAccount.availableAmount.toNumber()).to.eq(0);
    expect(memberAccount.stakeAmount.toNumber()).to.eq(100);
  });

  it("waits", async () => {
    await sleep(2500);
  });

  it("claims", async () => {
    await claimReward(ctx, 2, ctx.user1);

    expect(await (await ctx.rewardATA(ctx.user1.publicKey)).amount(ctx)).to.eq(
      97
    );
    expect(await ctx.factoryVault.amount(ctx)).to.eq(3);
  });
});
