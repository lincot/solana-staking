import { Context } from "./ctx";
import {
  PublicKey,
  sendAndConfirmTransaction,
  SystemProgram,
  Transaction,
} from "@solana/web3.js";

export async function airdrop(
  ctx: Context,
  addresses: PublicKey[]
): Promise<void> {
  await ctx.connection.confirmTransaction(
    await ctx.connection.requestAirdrop(
      ctx.payer.publicKey,
      200_000_000 * (addresses.length + 1)
    )
  );

  const tx = new Transaction();

  for (let i = 0; i < addresses.length; i++) {
    tx.add(
      SystemProgram.transfer({
        fromPubkey: ctx.payer.publicKey,
        lamports: 200_000_000,
        toPubkey: addresses[i],
      })
    );
  }

  await sendAndConfirmTransaction(ctx.connection, tx, [ctx.payer]);
}

export async function findPDA(
  ctx: Context,
  seeds: (Buffer | Uint8Array)[]
): Promise<PublicKey> {
  return (await PublicKey.findProgramAddress(seeds, ctx.program.programId))[0];
}

export function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
