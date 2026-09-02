import {
  createSignerFromKeypair,
  signerIdentity,
} from "@metaplex-foundation/umi";
import wallet from "/Users/akshaymeti/.config/solana/devnet-wallet.json";
import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import { irysUploader } from "@metaplex-foundation/umi-uploader-irys";

const umi = createUmi(
  process.env.SOLANA_RPC_URL ?? "https://api.devnet.solana.com",
);

const keypair = umi.eddsa.createKeypairFromSecretKey(new Uint8Array(wallet));
const signer = createSignerFromKeypair(umi, keypair);

umi.use(
  irysUploader({
    address: "https://devnet.irys.xyz/",
  }),
);

umi.use(signerIdentity(signer));

(async () => {
  try {
    //change the image uri to your image uri obtained from nft_image.ts
    const image ="https://gateway.irys.xyz/4bUFaCsqQYW7uRCi3eT8W1qJwAdsdAB2A4m6hi44pDmx";

    const metadata = {
  name: "GODS SPINNER",
  description: "Ultra Rare glowing gold spinner with 280 HP NFT.",
  image,
  attributes: [
    {
      trait_type: "godlike",
      value: "legendary",
    },
  ],
};

const myUri = await umi.uploader.uploadJson(metadata);

console.log(`metadata uri: ${myUri}`);
  } catch (error) {
    console.log("error", error);
  }
})();
