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
import { sha256 } from "@coral-xyz/anchor/dist/cjs/utils";

describe("initialize", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const { connection } = provider;

  const program = anchor.workspace.TokenSaleAnchor as Program<TokenSaleAnchor>;

  it("Initializes the token sale", async () => {
    try {
      const seller = provider.wallet.publicKey;
      const buyer = new Keypair();
      const mint = await createMint(provider);
      const temp_token_account = await createTokenAccount(
        provider,
        seller,
        mint,
        100_000 * LAMPORTS_PER_SOL
      );

      const { tokenSale, tokenSaleTokenAcctAuthority, buyerWhitelistAccount } =
        await getPDAs({
          programId: program.programId,
          seller,
          buyer: buyer.publicKey,
        });

      const initializeTransaction = await program.methods
        .initialize(new anchor.BN(2), new anchor.BN(100))
        .accounts({
          seller,
          tempTokenAccount: temp_token_account,
        })
        .rpc(COMMITMENT);
      console.log(`[Initialize] ${initializeTransaction}`);

      // Check data
      const tokenSaleData = await program.account.tokenSale.fetch(tokenSale);
      expect(tokenSaleData.sellerPubkey.toBase58()).to.eq(seller.toBase58());
      expect(tokenSaleData.tempTokenAccountPubkey.toBase58()).to.eq(
        temp_token_account.toBase58()
      );
      expect(tokenSaleData.perTokenPrice.toNumber()).to.eq(2);
      expect(tokenSaleData.purchaseLimit.toNumber()).to.eq(100);
    } catch (error) {
      console.error(error);
      throw new error();
    }
  });
  //
  //it("Whitelists a buyer for the token sale", async () => {
  //  try {
  //    const seller = provider.wallet.publicKey;
  //    const buyer = new Keypair();
  //
  //    const { tokenSale, tokenSaleTokenAcctAuthority, buyerWhitelistAccount } =
  //      await getPDAs({
  //        programId: program.programId,
  //        seller,
  //        buyer: buyer.publicKey,
  //      });
  //
  //    const whitelistUserTransaction = await program.methods
  //      .whitelistUser()
  //      .accounts({
  //        seller,
  //        buyer: buyer.publicKey,
  //      })
  //      .rpc(COMMITMENT);
  //    console.log(`[whitelist] ${whitelistUserTransaction}`);
  //
  //    // Check data
  //    const whitelistData = await program.account.whitelistData.fetch(
  //      buyerWhitelistAccount
  //    );
  //    expect(whitelistData.isWhitelisted).to.eq(true);
  //  } catch (error) {
  //    console.error(error);
  //    throw new error();
  //  }
  //});
});
