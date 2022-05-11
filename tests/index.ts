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

chai.use(chaiAsPromised);

const ctx = new Context();

before(async () => {
  await ctx.setup();
});

describe("instructions", () => {
  it("initialize", async () => {
    await initialize(ctx);

    const factory = await ctx.program.account.factory.fetch(ctx.factory);
    expect(factory.bump).to.be.above(200);
    expect(factory.authority).to.eql(ctx.factoryAuthority.publicKey);
  });

  it("createStaking", async () => {
    const unstakeTimelock = 10;

    await expect(
      createStaking(ctx, unstakeTimelock, {
        interestRate: { num: new BN(1), denom: new BN(0) },
      })
    ).to.be.rejectedWith("Zero");
    await expect(
      createStaking(ctx, unstakeTimelock, {
        proportional: { totalAmount: new BN(1), rewardPeriod: new BN(0) },
      })
    ).to.be.rejectedWith("Zero");
    await expect(
      createStaking(ctx, unstakeTimelock, {
        fixed: {
          requiredAmount: new BN(1),
          requiredPeriod: new BN(0),
          rewardAmount: new BN(1),
        },
      })
    ).to.be.rejectedWith("Zero");

    const rewardParams = {
      interestRate: { num: new BN(1337), denom: new BN(100) },
    };
    await createStaking(ctx, unstakeTimelock, rewardParams);

    const staking = await ctx.program.account.staking.fetch(
      await ctx.staking()
    );
    expect(staking.bump).to.be.above(200);
    expect(staking.authority).to.eql(ctx.stakingAuthority.publicKey);
    expect(staking.id).to.eql(0);
    expect(staking.stakeMint).to.eql(ctx.stakeMint);
    expect(staking.rewardMint).to.eql(ctx.rewardMint);
    expect(staking.unstakeTimelock).to.eql(unstakeTimelock);
    // @ts-ignore
    expect(staking.rewardParams.interestRate.num.toNumber()).to.eql(
      rewardParams.interestRate.num.toNumber()
    );
    // @ts-ignore
    expect(staking.rewardParams.interestRate.denom.toNumber()).to.eql(
      rewardParams.interestRate.denom.toNumber()
    );

    const configHistory = await ctx.program.account.configHistory.fetch(
      await ctx.configHistory()
    );
    expect(configHistory.bump).to.be.above(200);
    expect(configHistory.len).to.eql(1);
    expect(configHistory.rewardParams[0].interestRate.num.toNumber()).to.eql(
      rewardParams.interestRate.num.toNumber()
    );
    expect(configHistory.rewardParams[0].interestRate.denom.toNumber()).to.eql(
      rewardParams.interestRate.denom.toNumber()
    );
    expect(configHistory.startTimestamps[0]).to.not.eql(0);

    const stakesHistory = await ctx.program.account.stakesHistory.fetch(
      await ctx.stakesHistory()
    );
    expect(stakesHistory.bump).to.be.above(200);

    const factory = await ctx.program.account.factory.fetch(ctx.factory);
    expect(factory.stakingsCount).to.eql(1);
  });

  it("changeConfig", async () => {
    await changeConfig(ctx, null);

    await expect(
      changeConfig(ctx, {
        proportional: { totalAmount: new BN(1), rewardPeriod: new BN(1) },
      })
    ).to.be.rejectedWith("CannotChangeStakingType");
    await expect(
      changeConfig(ctx, {
        fixed: {
          requiredAmount: new BN(1),
          requiredPeriod: new BN(0),
          rewardAmount: new BN(1),
        },
      })
    ).to.be.rejectedWith("CannotChangeStakingType");

    await expect(
      changeConfig(ctx, {
        interestRate: { num: new BN(1), denom: new BN(0) },
      })
    ).to.be.rejectedWith("Zero");

    const rewardParams = {
      interestRate: {
        num: new BN(10),
        denom: new BN(100),
      },
    };
    await changeConfig(ctx, rewardParams);

    const staking = await ctx.program.account.staking.fetch(
      await ctx.staking()
    );
    // @ts-ignore
    expect(staking.rewardParams.interestRate.num.toNumber()).to.eql(
      rewardParams.interestRate.num.toNumber()
    );
    // @ts-ignore
    expect(staking.rewardParams.interestRate.denom.toNumber()).to.eql(
      rewardParams.interestRate.denom.toNumber()
    );

    const configHistory = await ctx.program.account.configHistory.fetch(
      await ctx.configHistory()
    );
    expect(configHistory.len).to.eql(2);
    expect(configHistory.rewardParams[1].interestRate.num.toNumber()).to.eql(
      rewardParams.interestRate.num.toNumber()
    );
    expect(configHistory.rewardParams[1].interestRate.denom.toNumber()).to.eql(
      rewardParams.interestRate.denom.toNumber()
    );
    expect(configHistory.startTimestamps[1]).to.not.eql(0);
  });

  it("registerMember", async () => {
    await registerMember(ctx, ctx.user1);

    const member = await ctx.program.account.member.fetch(
      await ctx.member(ctx.user1.publicKey)
    );
    expect(member.bump).to.be.above(200);
  });

  it("deposit", async () => {
    await deposit(ctx, ctx.user1, 100);

    expect(
      await (
        await ctx.stakeATA(await ctx.member(ctx.user1.publicKey))
      ).amount(ctx)
    ).to.eql(100);

    const member = await ctx.program.account.member.fetch(
      await ctx.member(ctx.user1.publicKey)
    );
    expect(member.availableAmount.toNumber()).to.eql(100);
  });

  it("stake", async () => {
    await expect(stake(ctx, ctx.user1, 101)).to.be.rejectedWith(
      "InsufficientBalance"
    );

    await stake(ctx, ctx.user1, 100);

    const member = await ctx.program.account.member.fetch(
      await ctx.member(ctx.user1.publicKey)
    );
    expect(member.availableAmount.toNumber()).to.eql(0);
    expect(member.stakeAmount.toNumber()).to.eql(100);

    const staking = await ctx.program.account.staking.fetch(
      await ctx.staking()
    );
    expect(staking.stakesSum.toNumber()).to.eql(100);
  });

  it("claimReward", async () => {
    await sleep(4000);

    await claimReward(ctx, ctx.user1);

    expect(
      await (await ctx.rewardATA(ctx.user1.publicKey)).amount(ctx)
    ).to.be.oneOf([39, 49, 59]);
    expect(await ctx.factoryVault.amount(ctx)).to.eql(1);

    const member = await ctx.program.account.member.fetch(
      await ctx.member(ctx.user1.publicKey)
    );
    expect(member.rewardsAmount.toNumber()).to.eql(0);
  });

  it("startUnstake", async () => {
    await expect(endUnstake(ctx, ctx.user1)).to.be.rejectedWith(
      "UnstakeInactive"
    );

    await expect(startUnstake(ctx, ctx.user1, 101)).to.be.rejectedWith(
      "InsufficientBalance"
    );

    await startUnstake(ctx, ctx.user1, 100);

    const member = await ctx.program.account.member.fetch(
      await ctx.member(ctx.user1.publicKey)
    );
    expect(member.pendingUnstakeActive).to.eql(true);
    expect(member.pendingUnstakeEndTs).to.not.eql(0);
    expect(member.stakeAmount.toNumber()).to.eql(0);
    expect(member.pendingAmount.toNumber()).to.eql(100);

    const staking = await ctx.program.account.staking.fetch(
      await ctx.staking()
    );
    expect(staking.stakesSum.toNumber()).to.eql(100);

    await expect(startUnstake(ctx, ctx.user1, 0)).to.be.rejectedWith(
      "UnstakeActive"
    );
  });

  it("endUnstake", async () => {
    await expect(endUnstake(ctx, ctx.user1)).to.be.rejectedWith(
      "UnstakeTimelock"
    );
    await sleep(10000);

    await endUnstake(ctx, ctx.user1);

    const member = await ctx.program.account.member.fetch(
      await ctx.member(ctx.user1.publicKey)
    );
    expect(member.availableAmount.toNumber()).to.eql(100);
    expect(member.pendingAmount.toNumber()).to.eql(0);
    expect(member.pendingUnstakeActive).to.eql(false);
  });

  it("withdraw", async () => {
    await expect(withdraw(ctx, ctx.user1, 101)).to.be.rejectedWith(
      "InsufficientBalance"
    );

    await withdraw(ctx, ctx.user1, 100);

    expect(await (await ctx.stakeATA(ctx.user1.publicKey)).amount(ctx)).to.eql(
      100
    );

    const member = await ctx.program.account.member.fetch(
      await ctx.member(ctx.user1.publicKey)
    );
    expect(member.availableAmount.toNumber()).to.eql(0);
  });

  after(async () => {
    await ctx.teardown();
  });
});

describe("interest rate", () => {
  it("creates staking", async () => {
    await createStaking(ctx, 0, {
      interestRate: { num: new BN(10), denom: new BN(100) },
    });
  });

  it("registers", async () => {
    await registerMember(ctx, ctx.user1);
  });

  it("deposits", async () => {
    await deposit(ctx, ctx.user1, 100);

    expect(
      await (
        await ctx.stakeATA(await ctx.member(ctx.user1.publicKey))
      ).amount(ctx)
    ).to.eql(100);

    const member = await ctx.program.account.member.fetch(
      await ctx.member(ctx.user1.publicKey)
    );
    expect(member.availableAmount.toNumber()).to.eql(100);
    expect(member.stakeAmount.toNumber()).to.eql(0);
    expect(member.pendingAmount.toNumber()).to.eql(0);
    expect(member.rewardsAmount.toNumber()).to.eql(0);
  });

  it("stakes", async () => {
    await stake(ctx, ctx.user1, 100);

    expect(
      await (
        await ctx.stakeATA(await ctx.member(ctx.user1.publicKey))
      ).amount(ctx)
    ).to.eql(100);

    const member = await ctx.program.account.member.fetch(
      await ctx.member(ctx.user1.publicKey)
    );
    expect(member.availableAmount.toNumber()).to.eql(0);
    expect(member.stakeAmount.toNumber()).to.eql(100);
    expect(member.pendingAmount.toNumber()).to.eql(0);
    expect(member.rewardsAmount.toNumber()).to.eql(0);
  });

  it("claims", async () => {
    await sleep(4000);

    await claimReward(ctx, ctx.user1);

    expect(
      await (await ctx.rewardATA(ctx.user1.publicKey)).amount(ctx)
    ).to.be.oneOf([39, 49, 59]);
    expect(await ctx.factoryVault.amount(ctx)).to.eql(1);

    expect(
      await (
        await ctx.stakeATA(await ctx.member(ctx.user1.publicKey))
      ).amount(ctx)
    ).to.eql(100);

    const member = await ctx.program.account.member.fetch(
      await ctx.member(ctx.user1.publicKey)
    );
    expect(member.availableAmount.toNumber()).to.eql(0);
    expect(member.stakeAmount.toNumber()).to.eql(100);
    expect(member.pendingAmount.toNumber()).to.eql(0);
    expect(member.rewardsAmount.toNumber()).to.eql(0);
  });

  it("starts unstake", async () => {
    await startUnstake(ctx, ctx.user1, 100);

    const member = await ctx.program.account.member.fetch(
      await ctx.member(ctx.user1.publicKey)
    );
    expect(member.availableAmount.toNumber()).to.eql(0);
    expect(member.stakeAmount.toNumber()).to.eql(0);
    expect(member.pendingAmount.toNumber()).to.eql(100);
    expect(member.rewardsAmount.toNumber()).to.eql(0);
  });

  it("ends unstake", async () => {
    await endUnstake(ctx, ctx.user1);

    const member = await ctx.program.account.member.fetch(
      await ctx.member(ctx.user1.publicKey)
    );
    expect(member.availableAmount.toNumber()).to.eql(100);
    expect(member.stakeAmount.toNumber()).to.eql(0);
    expect(member.pendingAmount.toNumber()).to.eql(0);
    expect(member.rewardsAmount.toNumber()).to.eql(0);
  });

  it("withdraws", async () => {
    await withdraw(ctx, ctx.user1, 100);

    expect(await (await ctx.stakeATA(ctx.user1.publicKey)).amount(ctx)).to.eql(
      100
    );
    expect(
      await (
        await ctx.stakeATA(await ctx.member(ctx.user1.publicKey))
      ).amount(ctx)
    ).to.eql(0);

    const member = await ctx.program.account.member.fetch(
      await ctx.member(ctx.user1.publicKey)
    );
    expect(member.availableAmount.toNumber()).to.eql(0);
    expect(member.stakeAmount.toNumber()).to.eql(0);
    expect(member.pendingAmount.toNumber()).to.eql(0);
    expect(member.rewardsAmount.toNumber()).to.eql(0);
  });

  after(async () => {
    await ctx.teardown();
  });
});

describe("proportional (1 user)", () => {
  it("creates staking", async () => {
    await createStaking(ctx, 0, {
      proportional: { totalAmount: new BN(100), rewardPeriod: 10 },
    });
  });

  it("registers", async () => {
    await registerMember(ctx, ctx.user1);
  });

  it("deposits", async () => {
    await deposit(ctx, ctx.user1, 100);

    expect(
      await (
        await ctx.stakeATA(await ctx.member(ctx.user1.publicKey))
      ).amount(ctx)
    ).to.eql(100);

    const member = await ctx.program.account.member.fetch(
      await ctx.member(ctx.user1.publicKey)
    );
    expect(member.availableAmount.toNumber()).to.eql(100);
    expect(member.stakeAmount.toNumber()).to.eql(0);
    expect(member.pendingAmount.toNumber()).to.eql(0);
    expect(member.rewardsAmount.toNumber()).to.eql(0);
  });

  it("stakes", async () => {
    await stake(ctx, ctx.user1, 100);

    expect(
      await (
        await ctx.stakeATA(await ctx.member(ctx.user1.publicKey))
      ).amount(ctx)
    ).to.eql(100);

    const member = await ctx.program.account.member.fetch(
      await ctx.member(ctx.user1.publicKey)
    );
    expect(member.availableAmount.toNumber()).to.eql(0);
    expect(member.stakeAmount.toNumber()).to.eql(100);
    expect(member.pendingAmount.toNumber()).to.eql(0);
    expect(member.rewardsAmount.toNumber()).to.eql(0);
  });

  it("claims", async () => {
    await sleep(10000);

    await claimReward(ctx, ctx.user1);

    expect(await (await ctx.rewardATA(ctx.user1.publicKey)).amount(ctx)).to.eql(
      97
    );
    expect(await ctx.factoryVault.amount(ctx)).to.eql(3);

    expect(
      await (
        await ctx.stakeATA(await ctx.member(ctx.user1.publicKey))
      ).amount(ctx)
    ).to.eql(100);

    const member = await ctx.program.account.member.fetch(
      await ctx.member(ctx.user1.publicKey)
    );
    expect(member.availableAmount.toNumber()).to.eql(0);
    expect(member.stakeAmount.toNumber()).to.eql(100);
    expect(member.pendingAmount.toNumber()).to.eql(0);
    expect(member.rewardsAmount.toNumber()).to.eql(0);
  });

  it("starts unstake", async () => {
    await startUnstake(ctx, ctx.user1, 100);

    const member = await ctx.program.account.member.fetch(
      await ctx.member(ctx.user1.publicKey)
    );
    expect(member.availableAmount.toNumber()).to.eql(0);
    expect(member.stakeAmount.toNumber()).to.eql(0);
    expect(member.pendingAmount.toNumber()).to.eql(100);
    expect(member.rewardsAmount.toNumber()).to.eql(0);
  });

  it("ends unstake", async () => {
    await endUnstake(ctx, ctx.user1);

    const member = await ctx.program.account.member.fetch(
      await ctx.member(ctx.user1.publicKey)
    );
    expect(member.availableAmount.toNumber()).to.eql(100);
    expect(member.stakeAmount.toNumber()).to.eql(0);
    expect(member.pendingAmount.toNumber()).to.eql(0);
    expect(member.rewardsAmount.toNumber()).to.eql(0);
  });

  it("withdraws", async () => {
    await withdraw(ctx, ctx.user1, 100);

    expect(await (await ctx.stakeATA(ctx.user1.publicKey)).amount(ctx)).to.eql(
      100
    );

    const member = await ctx.program.account.member.fetch(
      await ctx.member(ctx.user1.publicKey)
    );
    expect(member.availableAmount.toNumber()).to.eql(0);
    expect(member.stakeAmount.toNumber()).to.eql(0);
    expect(member.pendingAmount.toNumber()).to.eql(0);
    expect(member.rewardsAmount.toNumber()).to.eql(0);
  });

  after(async () => {
    await ctx.teardown();
  });
});

describe("proportional (1 user, changing config)", () => {
  it("creates staking", async () => {
    await createStaking(ctx, 0, {
      proportional: { totalAmount: new BN(100), rewardPeriod: 10 },
    });
  });

  it("registers", async () => {
    await registerMember(ctx, ctx.user1);
  });

  it("deposits", async () => {
    await deposit(ctx, ctx.user1, 100);

    expect(
      await (
        await ctx.stakeATA(await ctx.member(ctx.user1.publicKey))
      ).amount(ctx)
    ).to.eql(100);

    const member = await ctx.program.account.member.fetch(
      await ctx.member(ctx.user1.publicKey)
    );
    expect(member.availableAmount.toNumber()).to.eql(100);
    expect(member.stakeAmount.toNumber()).to.eql(0);
    expect(member.pendingAmount.toNumber()).to.eql(0);
    expect(member.rewardsAmount.toNumber()).to.eql(0);
  });

  it("stakes", async () => {
    await stake(ctx, ctx.user1, 100);

    expect(
      await (
        await ctx.stakeATA(await ctx.member(ctx.user1.publicKey))
      ).amount(ctx)
    ).to.eql(100);

    const member = await ctx.program.account.member.fetch(
      await ctx.member(ctx.user1.publicKey)
    );
    expect(member.availableAmount.toNumber()).to.eql(0);
    expect(member.stakeAmount.toNumber()).to.eql(100);
    expect(member.pendingAmount.toNumber()).to.eql(0);
    expect(member.rewardsAmount.toNumber()).to.eql(0);
  });

  it("changes config", async () => {
    await sleep(7000);

    await changeConfig(ctx, {
      proportional: { totalAmount: new BN(200), rewardPeriod: 10 },
    });
  });

  it("claims", async () => {
    await sleep(10000);

    await claimReward(ctx, ctx.user1);

    expect(await (await ctx.rewardATA(ctx.user1.publicKey)).amount(ctx)).to.eql(
      291
    );
    expect(await ctx.factoryVault.amount(ctx)).to.eql(9);

    expect(
      await (
        await ctx.stakeATA(await ctx.member(ctx.user1.publicKey))
      ).amount(ctx)
    ).to.eql(100);

    const member = await ctx.program.account.member.fetch(
      await ctx.member(ctx.user1.publicKey)
    );
    expect(member.availableAmount.toNumber()).to.eql(0);
    expect(member.stakeAmount.toNumber()).to.eql(100);
    expect(member.pendingAmount.toNumber()).to.eql(0);
    expect(member.rewardsAmount.toNumber()).to.eql(0);
  });

  it("starts unstake", async () => {
    await startUnstake(ctx, ctx.user1, 100);

    const member = await ctx.program.account.member.fetch(
      await ctx.member(ctx.user1.publicKey)
    );
    expect(member.availableAmount.toNumber()).to.eql(0);
    expect(member.stakeAmount.toNumber()).to.eql(0);
    expect(member.pendingAmount.toNumber()).to.eql(100);
    expect(member.rewardsAmount.toNumber()).to.eql(0);
  });

  it("ends unstake", async () => {
    await endUnstake(ctx, ctx.user1);

    const member = await ctx.program.account.member.fetch(
      await ctx.member(ctx.user1.publicKey)
    );
    expect(member.availableAmount.toNumber()).to.eql(100);
    expect(member.stakeAmount.toNumber()).to.eql(0);
    expect(member.pendingAmount.toNumber()).to.eql(0);
    expect(member.rewardsAmount.toNumber()).to.eql(0);
  });

  it("withdraws", async () => {
    await withdraw(ctx, ctx.user1, 100);

    expect(await (await ctx.stakeATA(ctx.user1.publicKey)).amount(ctx)).to.eql(
      100
    );

    const member = await ctx.program.account.member.fetch(
      await ctx.member(ctx.user1.publicKey)
    );
    expect(member.availableAmount.toNumber()).to.eql(0);
    expect(member.stakeAmount.toNumber()).to.eql(0);
    expect(member.pendingAmount.toNumber()).to.eql(0);
    expect(member.rewardsAmount.toNumber()).to.eql(0);
  });

  after(async () => {
    await ctx.teardown();
  });
});

describe("proportional (2 users)", () => {
  it("creates staking", async () => {
    await createStaking(ctx, 0, {
      proportional: { totalAmount: new BN(100), rewardPeriod: 10 },
    });
  });

  it("registers", async () => {
    await registerMember(ctx, ctx.user1);
    await registerMember(ctx, ctx.user2);
  });

  it("deposits", async () => {
    await deposit(ctx, ctx.user1, 100);
    await deposit(ctx, ctx.user2, 100);

    expect(
      await (
        await ctx.stakeATA(await ctx.member(ctx.user1.publicKey))
      ).amount(ctx)
    ).to.eql(100);

    const member1 = await ctx.program.account.member.fetch(
      await ctx.member(ctx.user1.publicKey)
    );
    expect(member1.availableAmount.toNumber()).to.eql(100);
    expect(member1.stakeAmount.toNumber()).to.eql(0);
    expect(member1.pendingAmount.toNumber()).to.eql(0);
    expect(member1.rewardsAmount.toNumber()).to.eql(0);

    expect(
      await (
        await ctx.stakeATA(await ctx.member(ctx.user2.publicKey))
      ).amount(ctx)
    ).to.eql(100);

    const member2 = await ctx.program.account.member.fetch(
      await ctx.member(ctx.user2.publicKey)
    );
    expect(member2.availableAmount.toNumber()).to.eql(100);
    expect(member2.stakeAmount.toNumber()).to.eql(0);
    expect(member2.pendingAmount.toNumber()).to.eql(0);
    expect(member2.rewardsAmount.toNumber()).to.eql(0);
  });

  it("stakes", async () => {
    await stake(ctx, ctx.user1, 100);
    await stake(ctx, ctx.user2, 100);

    expect(
      await (
        await ctx.stakeATA(await ctx.member(ctx.user1.publicKey))
      ).amount(ctx)
    ).to.eql(100);

    const member = await ctx.program.account.member.fetch(
      await ctx.member(ctx.user1.publicKey)
    );
    expect(member.availableAmount.toNumber()).to.eql(0);
    expect(member.stakeAmount.toNumber()).to.eql(100);
    expect(member.pendingAmount.toNumber()).to.eql(0);
    expect(member.rewardsAmount.toNumber()).to.eql(0);

    expect(
      await (
        await ctx.stakeATA(await ctx.member(ctx.user2.publicKey))
      ).amount(ctx)
    ).to.eql(100);

    const member2 = await ctx.program.account.member.fetch(
      await ctx.member(ctx.user2.publicKey)
    );
    expect(member2.availableAmount.toNumber()).to.eql(0);
    expect(member2.stakeAmount.toNumber()).to.eql(100);
    expect(member2.pendingAmount.toNumber()).to.eql(0);
    expect(member2.rewardsAmount.toNumber()).to.eql(0);
  });

  it("claims", async () => {
    await sleep(10000);

    await claimReward(ctx, ctx.user1);
    await claimReward(ctx, ctx.user2);

    expect(await (await ctx.rewardATA(ctx.user1.publicKey)).amount(ctx)).to.eql(
      49
    );
    expect(await (await ctx.rewardATA(ctx.user2.publicKey)).amount(ctx)).to.eql(
      49
    );
    expect(await ctx.factoryVault.amount(ctx)).to.eql(2);

    expect(
      await (
        await ctx.stakeATA(await ctx.member(ctx.user1.publicKey))
      ).amount(ctx)
    ).to.eql(100);

    const member = await ctx.program.account.member.fetch(
      await ctx.member(ctx.user1.publicKey)
    );
    expect(member.availableAmount.toNumber()).to.eql(0);
    expect(member.stakeAmount.toNumber()).to.eql(100);
    expect(member.pendingAmount.toNumber()).to.eql(0);
    expect(member.rewardsAmount.toNumber()).to.eql(0);

    expect(
      await (
        await ctx.stakeATA(await ctx.member(ctx.user2.publicKey))
      ).amount(ctx)
    ).to.eql(100);

    const member2 = await ctx.program.account.member.fetch(
      await ctx.member(ctx.user2.publicKey)
    );
    expect(member2.availableAmount.toNumber()).to.eql(0);
    expect(member2.stakeAmount.toNumber()).to.eql(100);
    expect(member2.pendingAmount.toNumber()).to.eql(0);
    expect(member2.rewardsAmount.toNumber()).to.eql(0);
  });

  it("starts unstake", async () => {
    await startUnstake(ctx, ctx.user1, 100);
    await startUnstake(ctx, ctx.user2, 100);

    const member = await ctx.program.account.member.fetch(
      await ctx.member(ctx.user1.publicKey)
    );
    expect(member.availableAmount.toNumber()).to.eql(0);
    expect(member.stakeAmount.toNumber()).to.eql(0);
    expect(member.pendingAmount.toNumber()).to.eql(100);
    expect(member.rewardsAmount.toNumber()).to.eql(0);

    const member2 = await ctx.program.account.member.fetch(
      await ctx.member(ctx.user2.publicKey)
    );
    expect(member2.availableAmount.toNumber()).to.eql(0);
    expect(member2.stakeAmount.toNumber()).to.eql(0);
    expect(member2.pendingAmount.toNumber()).to.eql(100);
    expect(member2.rewardsAmount.toNumber()).to.eql(0);
  });

  it("ends unstake", async () => {
    await endUnstake(ctx, ctx.user1);
    await endUnstake(ctx, ctx.user2);

    const member = await ctx.program.account.member.fetch(
      await ctx.member(ctx.user1.publicKey)
    );
    expect(member.availableAmount.toNumber()).to.eql(100);
    expect(member.stakeAmount.toNumber()).to.eql(0);
    expect(member.pendingAmount.toNumber()).to.eql(0);
    expect(member.rewardsAmount.toNumber()).to.eql(0);

    const member2 = await ctx.program.account.member.fetch(
      await ctx.member(ctx.user2.publicKey)
    );
    expect(member2.availableAmount.toNumber()).to.eql(100);
    expect(member2.stakeAmount.toNumber()).to.eql(0);
    expect(member2.pendingAmount.toNumber()).to.eql(0);
    expect(member2.rewardsAmount.toNumber()).to.eql(0);
  });

  it("withdraws", async () => {
    await withdraw(ctx, ctx.user1, 100);
    await withdraw(ctx, ctx.user2, 100);

    expect(await (await ctx.stakeATA(ctx.user1.publicKey)).amount(ctx)).to.eql(
      100
    );

    const member = await ctx.program.account.member.fetch(
      await ctx.member(ctx.user1.publicKey)
    );
    expect(member.availableAmount.toNumber()).to.eql(0);
    expect(member.stakeAmount.toNumber()).to.eql(0);
    expect(member.pendingAmount.toNumber()).to.eql(0);
    expect(member.rewardsAmount.toNumber()).to.eql(0);

    expect(await (await ctx.stakeATA(ctx.user2.publicKey)).amount(ctx)).to.eql(
      100
    );

    const member2 = await ctx.program.account.member.fetch(
      await ctx.member(ctx.user2.publicKey)
    );
    expect(member2.availableAmount.toNumber()).to.eql(0);
    expect(member2.stakeAmount.toNumber()).to.eql(0);
    expect(member2.pendingAmount.toNumber()).to.eql(0);
    expect(member2.rewardsAmount.toNumber()).to.eql(0);
  });

  after(async () => {
    await ctx.teardown();
  });
});

describe("fixed", () => {
  it("creates staking", async () => {
    await createStaking(ctx, 0, {
      fixed: {
        requiredAmount: new BN(100),
        requiredPeriod: 10,
        rewardAmount: new BN(100),
      },
    });
  });

  it("registers", async () => {
    await registerMember(ctx, ctx.user1);
  });

  it("deposits", async () => {
    await deposit(ctx, ctx.user1, 100);

    expect(
      await (
        await ctx.stakeATA(await ctx.member(ctx.user1.publicKey))
      ).amount(ctx)
    ).to.eql(100);

    const member = await ctx.program.account.member.fetch(
      await ctx.member(ctx.user1.publicKey)
    );
    expect(member.availableAmount.toNumber()).to.eql(100);
    expect(member.stakeAmount.toNumber()).to.eql(0);
    expect(member.pendingAmount.toNumber()).to.eql(0);
    expect(member.rewardsAmount.toNumber()).to.eql(0);
  });

  it("stakes", async () => {
    await stake(ctx, ctx.user1, 100);

    expect(
      await (
        await ctx.stakeATA(await ctx.member(ctx.user1.publicKey))
      ).amount(ctx)
    ).to.eql(100);

    const member = await ctx.program.account.member.fetch(
      await ctx.member(ctx.user1.publicKey)
    );
    expect(member.availableAmount.toNumber()).to.eql(0);
    expect(member.stakeAmount.toNumber()).to.eql(100);
    expect(member.pendingAmount.toNumber()).to.eql(0);
    expect(member.rewardsAmount.toNumber()).to.eql(0);
  });

  it("claims", async () => {
    await sleep(10000);

    await claimReward(ctx, ctx.user1);

    expect(await (await ctx.rewardATA(ctx.user1.publicKey)).amount(ctx)).to.eql(
      97
    );
    expect(await ctx.factoryVault.amount(ctx)).to.eql(3);

    expect(
      await (
        await ctx.stakeATA(await ctx.member(ctx.user1.publicKey))
      ).amount(ctx)
    ).to.eql(100);

    const member = await ctx.program.account.member.fetch(
      await ctx.member(ctx.user1.publicKey)
    );
    expect(member.availableAmount.toNumber()).to.eql(0);
    expect(member.stakeAmount.toNumber()).to.eql(100);
    expect(member.pendingAmount.toNumber()).to.eql(0);
    expect(member.rewardsAmount.toNumber()).to.eql(0);
  });

  it("starts unstake", async () => {
    await startUnstake(ctx, ctx.user1, 100);

    const member = await ctx.program.account.member.fetch(
      await ctx.member(ctx.user1.publicKey)
    );
    expect(member.availableAmount.toNumber()).to.eql(0);
    expect(member.stakeAmount.toNumber()).to.eql(0);
    expect(member.pendingAmount.toNumber()).to.eql(100);
    expect(member.rewardsAmount.toNumber()).to.eql(0);
  });

  it("ends unstake", async () => {
    await endUnstake(ctx, ctx.user1);

    const member = await ctx.program.account.member.fetch(
      await ctx.member(ctx.user1.publicKey)
    );
    expect(member.availableAmount.toNumber()).to.eql(100);
    expect(member.stakeAmount.toNumber()).to.eql(0);
    expect(member.pendingAmount.toNumber()).to.eql(0);
    expect(member.rewardsAmount.toNumber()).to.eql(0);
  });

  it("withdraws", async () => {
    await withdraw(ctx, ctx.user1, 100);

    expect(await (await ctx.stakeATA(ctx.user1.publicKey)).amount(ctx)).to.eql(
      100
    );

    const member = await ctx.program.account.member.fetch(
      await ctx.member(ctx.user1.publicKey)
    );
    expect(member.availableAmount.toNumber()).to.eql(0);
    expect(member.stakeAmount.toNumber()).to.eql(0);
    expect(member.pendingAmount.toNumber()).to.eql(0);
    expect(member.rewardsAmount.toNumber()).to.eql(0);
  });

  after(async () => {
    await ctx.teardown();
  });
});
