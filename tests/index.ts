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

  it("registers member", async () => {
    await registerMember(ctx, 0, ctx.beneficiary);
    await mintTo(
      ctx,
      await ctx.findATA(ctx.beneficiary.publicKey),
      ctx.mintAuthority,
      100
    );
  });

  it("deposits", async () => {
    await deposit(ctx, 0, ctx.beneficiary, 100);

    expect(
      await (await ctx.available(ctx.beneficiary.publicKey, 0)).amount(ctx)
    ).to.eql(100);
  });

  it("fails to claim reward before staking", async () => {
    await expect(claimReward(ctx, 0, ctx.beneficiary)).to.be.rejected;
  });

  it("stakes", async () => {
    await stake(ctx, 0, ctx.beneficiary, 100);

    expect(
      await (await ctx.available(ctx.beneficiary.publicKey, 0)).amount(ctx)
    ).to.eql(0);
    expect(
      await (await ctx.stake(ctx.beneficiary.publicKey, 0)).amount(ctx)
    ).to.eql(100);
  });

  it("waits", async () => {
    await sleep(3000);
  });

  it("claims reward", async () => {
    await claimReward(ctx, 0, ctx.beneficiary);

    expect(
      await (await ctx.findATA(ctx.beneficiary.publicKey)).amount(ctx)
    ).to.be.oneOf([39, 49, 59]);
    expect(await ctx.factoryVault.amount(ctx)).to.eql(1);
  });

  it("starts unstake", async () => {
    await startUnstake(ctx, 0, ctx.beneficiary, 100);

    expect(
      await (await ctx.stake(ctx.beneficiary.publicKey, 0)).amount(ctx)
    ).to.eql(0);
    expect(
      await (await ctx.pending(ctx.beneficiary.publicKey, 0)).amount(ctx)
    ).to.eql(100);
  });

  it("fails to end unstake before timelock", async () => {
    await expect(endUnstake(ctx, 0, ctx.beneficiary)).to.be.rejected;
  });

  it("waits for unstake timelock to end", async () => {
    await sleep(2000);
  });

  it("ends unstake", async () => {
    await endUnstake(ctx, 0, ctx.beneficiary);

    expect(
      await (await ctx.available(ctx.beneficiary.publicKey, 0)).amount(ctx)
    ).to.eql(100);
  });

  it("withdraws", async () => {
    await withdraw(ctx, 0, ctx.beneficiary);

    expect(
      await (await ctx.findATA(ctx.beneficiary.publicKey)).amount(ctx)
    ).to.be.oneOf([139, 149, 159]);
  });
});

describe("proportional", () => {
  it("creates staking", async () => {
    await createStaking(ctx, 2, {
      proportional: { totalAmount: new BN(100), rewardPeriod: 3 },
    });
    await burnAll(
      ctx,
      await ctx.findATA(ctx.factoryAuthority.publicKey),
      ctx.factoryAuthority
    );
  });

  it("registers member", async () => {
    await registerMember(ctx, 1, ctx.beneficiary);
    await burnAll(
      ctx,
      await ctx.findATA(ctx.beneficiary.publicKey),
      ctx.beneficiary
    );
    await mintTo(
      ctx,
      await ctx.findATA(ctx.beneficiary.publicKey),
      ctx.mintAuthority,
      100
    );
  });

  it("deposits", async () => {
    await deposit(ctx, 1, ctx.beneficiary, 100);

    expect(
      await (await ctx.available(ctx.beneficiary.publicKey, 1)).amount(ctx)
    ).to.eql(100);
  });

  it("fails to claim reward before staking", async () => {
    await expect(claimReward(ctx, 1, ctx.beneficiary)).to.be.rejected;
  });

  it("stakes", async () => {
    await stake(ctx, 1, ctx.beneficiary, 100);

    expect(
      await (await ctx.available(ctx.beneficiary.publicKey, 1)).amount(ctx)
    ).to.eql(0);
    expect(
      await (await ctx.stake(ctx.beneficiary.publicKey, 1)).amount(ctx)
    ).to.eql(100);
  });

  it("waits", async () => {
    await sleep(3000);
  });

  it("claims reward", async () => {
    await claimReward(ctx, 1, ctx.beneficiary);

    expect(
      await (await ctx.findATA(ctx.beneficiary.publicKey)).amount(ctx)
    ).to.eql(97);
    expect(await ctx.factoryVault.amount(ctx)).to.eql(3);
  });

  it("starts unstake", async () => {
    await startUnstake(ctx, 1, ctx.beneficiary, 100);

    expect(
      await (await ctx.stake(ctx.beneficiary.publicKey, 1)).amount(ctx)
    ).to.eql(0);
    expect(
      await (await ctx.pending(ctx.beneficiary.publicKey, 1)).amount(ctx)
    ).to.eql(100);
  });

  it("fails to end unstake before timelock", async () => {
    await expect(endUnstake(ctx, 1, ctx.beneficiary)).to.be.rejected;
  });

  it("waits for unstake timelock to end", async () => {
    await sleep(2000);
  });

  it("ends unstake", async () => {
    await endUnstake(ctx, 1, ctx.beneficiary);

    expect(
      await (await ctx.available(ctx.beneficiary.publicKey, 1)).amount(ctx)
    ).to.eql(100);
  });

  it("withdraws", async () => {
    await withdraw(ctx, 1, ctx.beneficiary);

    expect(
      await (await ctx.findATA(ctx.beneficiary.publicKey)).amount(ctx)
    ).to.eql(197);
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
    await burnAll(
      ctx,
      await ctx.findATA(ctx.factoryAuthority.publicKey),
      ctx.factoryAuthority
    );
  });

  it("registers member", async () => {
    await registerMember(ctx, 2, ctx.beneficiary);
    await burnAll(
      ctx,
      await ctx.findATA(ctx.beneficiary.publicKey),
      ctx.beneficiary
    );
    await mintTo(
      ctx,
      await ctx.findATA(ctx.beneficiary.publicKey),
      ctx.mintAuthority,
      100
    );
  });

  it("deposits", async () => {
    await deposit(ctx, 2, ctx.beneficiary, 100);

    expect(
      await (await ctx.available(ctx.beneficiary.publicKey, 2)).amount(ctx)
    ).to.eql(100);
  });

  it("fails to claim reward before staking", async () => {
    await expect(claimReward(ctx, 2, ctx.beneficiary)).to.be.rejected;
  });

  it("stakes", async () => {
    await stake(ctx, 2, ctx.beneficiary, 100);

    expect(
      await (await ctx.available(ctx.beneficiary.publicKey, 2)).amount(ctx)
    ).to.eql(0);
    expect(
      await (await ctx.stake(ctx.beneficiary.publicKey, 2)).amount(ctx)
    ).to.eql(100);
  });

  it("waits", async () => {
    await sleep(3000);
  });

  it("claims reward", async () => {
    await claimReward(ctx, 2, ctx.beneficiary);

    expect(
      await (await ctx.findATA(ctx.beneficiary.publicKey)).amount(ctx)
    ).to.eql(97);
    expect(await ctx.factoryVault.amount(ctx)).to.eql(3);
  });

  it("starts unstake", async () => {
    await startUnstake(ctx, 2, ctx.beneficiary, 100);

    expect(
      await (await ctx.stake(ctx.beneficiary.publicKey, 2)).amount(ctx)
    ).to.eql(0);
    expect(
      await (await ctx.pending(ctx.beneficiary.publicKey, 2)).amount(ctx)
    ).to.eql(100);
  });

  it("fails to end unstake before timelock", async () => {
    await expect(endUnstake(ctx, 2, ctx.beneficiary)).to.be.rejected;
  });

  it("waits for unstake timelock to end", async () => {
    await sleep(2000);
  });

  it("ends unstake", async () => {
    await endUnstake(ctx, 2, ctx.beneficiary);

    expect(
      await (await ctx.available(ctx.beneficiary.publicKey, 2)).amount(ctx)
    ).to.eql(100);
  });

  it("withdraws", async () => {
    await withdraw(ctx, 2, ctx.beneficiary);

    expect(
      await (await ctx.findATA(ctx.beneficiary.publicKey)).amount(ctx)
    ).to.eql(197);
  });
});
