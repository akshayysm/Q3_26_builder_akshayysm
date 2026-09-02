import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import wallet from "/Users/akshaymeti/.config/solana/devnet-wallet.json";
import {
  createSignerFromKeypair,
  signerIdentity,
  publicKey,
} from "@metaplex-foundation/umi";
import {mplCore,update,fetchAsset,} from "@metaplex-foundation/mpl-core";
import { base58 } from "@metaplex-foundation/umi/serializers";

const umi = createUmi(
  process.env.SOLANA_RPC_URL ?? "https://api.devnet.solana.com",
);

const keypair = umi.eddsa.createKeypairFromSecretKey(
  new Uint8Array(wallet),
);

const signer = createSignerFromKeypair(umi, keypair);

umi.use(signerIdentity(signer));
umi.use(mplCore());

const assetAddress = publicKey("jvCTVfHRsvr2XXcTbixipNubKYukdrHNCEfNc9dZvcM",);

(async () => {
  try {
    const asset = await fetchAsset(umi, assetAddress);

    const tx = await update(umi, {
        asset,
        name: "GODS SPINNER #1",
        uri: "https://gateway.irys.xyz/DjbT7BgyoCUxGsJsXaMLoAMc3oydZ8M8HtxVFuTDFXYD",
    }).sendAndConfirm(umi);

    const signature = base58.deserialize(tx.signature)[0];

    console.log(`Update signature: ${signature}`);
  } catch (error) {
    console.error("Update error:", error);
  }
})();