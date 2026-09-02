import assert from "node:assert/strict";
import { describe, it } from "mocha";

import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import {
  createSignerFromKeypair,
  publicKey,
  signerIdentity,
} from "@metaplex-foundation/umi";

import {
  fetchAsset,
  mplCore,
} from "@metaplex-foundation/mpl-core";

import {
  fetchDigitalAsset,
  mplTokenMetadata,
} from "@metaplex-foundation/mpl-token-metadata";

import wallet from "/Users/akshaymeti/.config/solana/devnet-wallet.json";

import {
  address,
  createSolanaRpc,
} from "@solana/kit";

import {
  findAssociatedTokenPda,
  TOKEN_PROGRAM_ADDRESS,
} from "@solana-program/token";


const umi = createUmi("https://api.devnet.solana.com",);

const keypair =
  umi.eddsa.createKeypairFromSecretKey(
    new Uint8Array(wallet),
  );

const signer = createSignerFromKeypair(
  umi,
  keypair,
);

umi.use(signerIdentity(signer));
umi.use(mplCore());
umi.use(mplTokenMetadata());

const rpc = createSolanaRpc(
  "https://api.devnet.solana.com",
);


const mint = publicKey("6r4h1EkCeEoFyTdXn9uKbWyLcyLPxXnBv45XHSwDhPR4");

const nft = publicKey("jvCTVfHRsvr2XXcTbixipNubKYukdrHNCEfNc9dZvcM");

const recipient = address("FdUzBqV6nCKgmeLm4XLRt2Sdj38iE5Uz6RaXofqYrhd");

const imageUri ="https://gateway.irys.xyz/4bUFaCsqQYW7uRCi3eT8W1qJwAdsdAB2A4m6hi44pDmx";

const metadataUri ="https://gateway.irys.xyz/DjbT7BgyoCUxGsJsXaMLoAMc3oydZ8M8HtxVFuTDFXYD";


describe("spl_nft_testsresult", () => {

  it("Initialize SPL token", async () => {
    const token = await fetchDigitalAsset(
      umi,
      mint,
    );

    assert.equal(
      token.mint.publicKey,
      mint,
    );
  });


  it("Add SPL token metadata", async () => {
    const token = await fetchDigitalAsset(
      umi,
      mint,
    );

    assert.equal(
      String(token.metadata.name),
      "FIDGET",
    );

    assert.equal(
      String(token.metadata.symbol),
      "SPINS",
    );
  });


  it("Mint SPL tokens", async () => {
    const supply = await rpc
      .getTokenSupply(address(mint))
      .send();

    assert.ok(
      BigInt(supply.value.amount) >= 1_000_000n,
    );
  });


  it("Transfer SPL tokens", async () => {
    const [recipientAta] =
      await findAssociatedTokenPda({
        mint: address(mint),
        owner: recipient,
        tokenProgram: TOKEN_PROGRAM_ADDRESS,
      });

    const balance = await rpc
      .getTokenAccountBalance(recipientAta)
      .send();

    assert.ok(
      BigInt(balance.value.amount) >= 1_000_000n,
    );
  });


  it("Upload NFT image", async () => {
    const response = await fetch(
      imageUri,
    );

    assert.equal(
      response.ok,
      true,
    );
  });


  it("Create NFT metadata", async () => {
    const response = await fetch(
      metadataUri,
    );

    assert.equal(
      response.ok,
      true,
    );

    const metadata =
      (await response.json()) as {
        name?: string;
        image?: string;
      };

    assert.equal(
      metadata.name,
      "GODS SPINNER",
    );

    assert.equal(
      metadata.image,
      imageUri,
    );
  });


  it("Mint MPL Core NFT", async () => {
    const asset = await fetchAsset(
      umi,
      nft,
    );

    assert.equal(
      asset.publicKey,
      nft,
    );
  });


    it("Update NFT name and metadata", async () => {
    const asset = await fetchAsset(umi, nft);

    assert.equal(asset.name, "GODS SPINNER #1");

    assert.equal(asset.uri, metadataUri);

    assert.equal(
      asset.updateAuthority.address,
      signer.publicKey.toString(),
    );
  });
});