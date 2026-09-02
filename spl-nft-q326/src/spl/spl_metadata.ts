import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { address, createSolanaRpc } from "@solana/kit";
import { publicKey, createUmi } from "@metaplex-foundation/umi";
import { findMetadataPda } from "@metaplex-foundation/mpl-token-metadata";
import {
  findAssociatedTokenPda,
  TOKEN_PROGRAM_ADDRESS,
} from "@solana-program/token";

const rpc = createSolanaRpc("https://api.devnet.solana.com");

const umi = createUmi();

const mint = address(
  "6r4h1EkCeEoFyTdXn9uKbWyLcyLPxXnBv45XHSwDhPR4",
);

const walletAddress = address("61cT4TxVoDZ3Sga4fU8CjpEqwkPuDwYSSH63rJzFResJ",
);

const recipientAddress = address("9EUd4VNcjMAysd7zQk3Q1a4tb28BYndLNBAQDiYnHJ64",
);

describe("SPL Token Assignment", () => {
  it("should have an initialized mint", async () => {
    const account = await rpc.getAccountInfo(mint).send();

    assert.ok(account.value);
  });

  it("should have metadata for the mint", async () => {
    const metadata = findMetadataPda(umi, {
      mint: publicKey(mint),
    });

    const metadataAccount = await rpc
      .getAccountInfo(address(metadata[0]))
      .send();

    assert.ok(metadataAccount.value);
  });

  it("should have the sender ATA with tokens", async () => {
    const [senderAta] = await findAssociatedTokenPda({
      mint,
      owner: walletAddress,
      tokenProgram: TOKEN_PROGRAM_ADDRESS,
    });

    const account = await rpc.getTokenAccountBalance(senderAta).send();

    assert.ok(account.value);
    assert.ok(Number(account.value.amount) > 0);
  });

  it("should have the recipient ATA with tokens", async () => {
    const [recipientAta] = await findAssociatedTokenPda({
      mint,
      owner: recipientAddress,
      tokenProgram: TOKEN_PROGRAM_ADDRESS,
    });

    const account = await rpc.getTokenAccountBalance(recipientAta).send();

    assert.ok(account.value);
    assert.ok(Number(account.value.amount) > 0);
  });
});