import {
  burn,
  getAccount,
  getOrCreateAssociatedTokenAccount,
} from "@solana/spl-token";
import { Keypair, PublicKey } from "@solana/web3.js";
import { Context } from "./ctx";
import * as token from "@solana/spl-token";

export class TokenAccount extends PublicKey {
  mint: PublicKey;

  constructor(address: PublicKey, mint: PublicKey) {
    super(address);
    this.mint = mint;
  }

  async amount(ctx: Context): Promise<number> {
    return Number((await getAccount(ctx.connection, this)).amount);
  }
}

export async function createMint(
  ctx: Context,
  authority: Keypair,
  decimals: number
): Promise<PublicKey> {
  return await token.createMint(
    ctx.connection,
    ctx.payer,
    authority.publicKey,
    undefined,
    decimals
  );
}

export async function mintTo(
  ctx: Context,
  destination: TokenAccount,
  mintAuthority: Keypair,
  amount: number | bigint
): Promise<void> {
  await token.mintTo(
    ctx.connection,
    ctx.payer,
    destination.mint,
    destination,
    mintAuthority,
    amount
  );
}

export async function findATA(
  ctx: Context,
  owner: PublicKey,
  mint: PublicKey
): Promise<TokenAccount> {
  const address = (
    await getOrCreateAssociatedTokenAccount(
      ctx.connection,
      ctx.payer,
      mint,
      owner,
      true
    )
  ).address;

  return new TokenAccount(address, mint);
}

export async function burnAll(
  ctx: Context,
  from: TokenAccount,
  owner: Keypair
): Promise<void> {
  await burn(
    ctx.connection,
    ctx.payer,
    from,
    from.mint,
    owner,
    await from.amount(ctx)
  );
}
