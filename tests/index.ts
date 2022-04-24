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

describe("staking", () => {
  const ctx = new Context();

  it("setups", async () => {
    await ctx.setup();
  });

  it("initializes", async () => {
    await initialize(ctx);
  });

  it("creates staking", async () => {
    await createStaking(ctx, 2, {
      interestRate: { num: new BN(1337), denom: new BN(100) },
    });
  });

  it("changes config", async () => {
    await changeConfig(ctx, {
      interestRate: { num: new BN(10), denom: new BN(100) },
    });
  });

  it("registers member", async () => {
    await registerMember(ctx, ctx.beneficiary);
  });

  it("deposits", async () => {
    await deposit(ctx, ctx.beneficiary, 100);

    expect(
      await (await ctx.available(ctx.beneficiary.publicKey)).amount(ctx)
    ).to.eql(BigInt(100));
  });

  it("fails to claim reward before staking", async () => {
    await expect(claimReward(ctx, ctx.beneficiary)).to.be.rejected;
  });

  it("stakes", async () => {
    await stake(ctx, ctx.beneficiary, 100);

    expect(
      await (await ctx.available(ctx.beneficiary.publicKey)).amount(ctx)
    ).to.eql(BigInt(0));
    expect(
      await (await ctx.stake(ctx.beneficiary.publicKey)).amount(ctx)
    ).to.eql(BigInt(100));
  });

  it("waits", async () => {
    await sleep(3000);
  });

  it("claims reward", async () => {
    await claimReward(ctx, ctx.beneficiary);

    expect(
      await (await ctx.findATA(ctx.beneficiary.publicKey)).amount(ctx)
    ).to.be.oneOf([BigInt(39), BigInt(49), BigInt(59)]);
    expect(await ctx.factoryVault.amount(ctx)).to.eql(BigInt(1));
  });

  it("starts unstake", async () => {
    await startUnstake(ctx, ctx.beneficiary, 100);

    expect(
      await (await ctx.stake(ctx.beneficiary.publicKey)).amount(ctx)
    ).to.eql(BigInt(0));
    expect(
      await (await ctx.pending(ctx.beneficiary.publicKey)).amount(ctx)
    ).to.eql(BigInt(100));
  });

  it("fails to end unstake before timelock", async () => {
    await expect(endUnstake(ctx, ctx.beneficiary)).to.be.rejected;
  });

  it("waits for unstake timelock to end", async () => {
    await sleep(2000);
  });

  it("ends unstake", async () => {
    await endUnstake(ctx, ctx.beneficiary);

    expect(
      await (await ctx.available(ctx.beneficiary.publicKey)).amount(ctx)
    ).to.eql(BigInt(100));
  });

  it("withdraws", async () => {
    await withdraw(ctx, ctx.beneficiary);

    expect(
      await (await ctx.findATA(ctx.beneficiary.publicKey)).amount(ctx)
    ).to.be.oneOf([BigInt(139), BigInt(149), BigInt(159)]);
  });
});
