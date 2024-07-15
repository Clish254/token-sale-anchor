import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import * as spl from "@solana/spl-token";

import { expect } from "chai";

import web3, { Keypair, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { TokenSaleAnchor } from "../target/types/token_sale_anchor";

import {
  COMMITMENT,
  PDAAccounts,
  ParsedTokenTransfer,
  createMint,
  createTokenAccount,
  getPDAs,
} from "./utils";

describe("whitelist", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const { connection } = provider;

  const program = anchor.workspace.TokenSaleAnchor as Program<TokenSaleAnchor>;

  it("Whitelists a buyer for the token sale", async () => {
    try {
      const seller = provider.wallet.publicKey;
      const buyer = new Keypair();
      const mint = await createMint(provider);
      const temp_token_account = await createTokenAccount(
        provider,
        provider.wallet.publicKey,
        mint,
        100_000 * LAMPORTS_PER_SOL
      );

      const { tokenSale, tokenSaleTokenAcctAuthority, buyerWhitelistAccount } =
        await getPDAs({
          programId: program.programId,
          seller,
          buyer: buyer.publicKey,
        });

      const whitelistUserTransaction = await program.methods
        .whitelistUser()
        .accounts({
          seller,
          buyer: buyer.publicKey,
        })
        .rpc(COMMITMENT);
      console.log(`[whitelist] ${whitelistUserTransaction}`);

      // Check data
      const whitelistData = await program.account.whitelistData.fetch(
        buyerWhitelistAccount
      );
      expect(whitelistData.isWhitelisted).to.eq(true);
    } catch (error) {
      console.error(error);
      throw new error();
    }
  });
});
