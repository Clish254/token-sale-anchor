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
  ParsedSolTransfer,
} from "./utils";
import { BN } from "bn.js";
import { SYSTEM_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/native/system";

describe("buy token", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const { connection } = provider;

  const program = anchor.workspace.TokenSaleAnchor as Program<TokenSaleAnchor>;

  it("Buys tokens from the token sale", async () => {
    try {
      const seller = provider.wallet.publicKey;
      const buyer = new Keypair();

      // Airdrop SOL to the buyer for the purchase
      await connection.requestAirdrop(buyer.publicKey, 10 * LAMPORTS_PER_SOL);
      await new Promise((resolve) => setTimeout(resolve, 2000));

      const mint = await createMint(provider);

      const temp_token_account = await createTokenAccount(
        provider,
        seller,
        mint,
        100_000 * LAMPORTS_PER_SOL
      );
      const buyerTokenAccount = await createTokenAccount(
        provider,
        buyer.publicKey,
        mint,
        0
      );

      const { tokenSale, tokenSaleTokenAcctAuthority, buyerWhitelistAccount } =
        await getPDAs({
          programId: program.programId,
          seller,
          buyer: buyer.publicKey,
        });

      const perTokenPrice = 2;
      const purchaseLimit = 100;
      const initializeTransaction = await program.methods
        .initialize(new anchor.BN(perTokenPrice), new anchor.BN(purchaseLimit))
        .accounts({
          seller,
          tempTokenAccount: temp_token_account,
        })
        .rpc(COMMITMENT);
      console.log(`[Initialize] ${initializeTransaction}`);

      // Whitelist the buyer
      const whitelistTransaction = await program.methods
        .whitelistUser()
        .accounts({
          seller,
          buyer: buyer.publicKey,
        })
        .rpc(COMMITMENT);
      console.log(`[Whitelist] ${whitelistTransaction}`);

      const numberOfTokens = 1;
      const buyTokenTransaction = await program.methods
        .buyToken(new anchor.BN(numberOfTokens))
        .accounts({
          buyer: buyer.publicKey,
          seller,
          buyerTokenAccount: buyerTokenAccount,
          tempTokenAccount: temp_token_account,
        })
        .signers([buyer])
        .rpc(COMMITMENT);
      console.log(`[Buy Token] ${buyTokenTransaction}`);

      // Check buyer's token account balance
      const buyerTokenAccountInfo = await spl.getAccount(
        connection,
        buyerTokenAccount
      );
      expect(Number(buyerTokenAccountInfo.amount)).to.eq(numberOfTokens);

      // Check temp token account balance
      const tempTokenAccountInfo = await spl.getAccount(
        connection,
        temp_token_account
      );
      expect(Number(tempTokenAccountInfo.amount)).to.eq(
        100_000 * LAMPORTS_PER_SOL - numberOfTokens
      );

      const tx = await connection.getParsedTransaction(
        buyTokenTransaction,
        COMMITMENT
      );

      // Ensure that inner SOL transfer succeded.
      const transferIx: any = tx.meta.innerInstructions[0].instructions.find(
        (ix) =>
          (ix as any).parsed.type === "transfer" &&
          ix.programId.toBase58() == SYSTEM_PROGRAM_ID.toBase58()
      );

      const parsedInfo: ParsedSolTransfer = transferIx.parsed.info;
      expect(parsedInfo).eql({
        lamports: numberOfTokens * perTokenPrice,
        destination: seller.toBase58(),
        source: buyer.publicKey.toBase58(),
      });
    } catch (error) {
      console.error(error);
      throw new error();
    }
  });
});
