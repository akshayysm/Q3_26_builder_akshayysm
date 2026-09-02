import {
  appendTransactionMessageInstruction,
  appendTransactionMessageInstructions,
  assertIsTransactionMessageWithBlockhashLifetime,
  assertIsTransactionWithBlockhashLifetime,
  createKeyPairSignerFromBytes,
  createSolanaRpc,
  createSolanaRpcSubscriptions,
  createTransactionMessage,
  generateKeyPairSigner,
  getSignatureFromTransaction,
  sendAndConfirmTransactionFactory,
  setTransactionMessageFeePayerSigner,
  setTransactionMessageLifetimeUsingBlockhash,
  signTransactionMessageWithSigners,
} from "@solana/kit";
import {
  getInitializeMintInstruction,
  getMintSize,
  TOKEN_PROGRAM_ADDRESS,
} from "@solana-program/token";
import { getCreateAccountInstruction } from "@solana-program/system";

import wallet from "/Users/akshaymeti/Desktop/spl-nft-q326/wallet.json";

const rpc = createSolanaRpc("https://api.devnet.solana.com");

const rpcSubscriptions = createSolanaRpcSubscriptions(
  "wss://api.devnet.solana.com",
);

(async () => {
  // create a signer from your wallet
  const signer = await createKeyPairSignerFromBytes(new Uint8Array(wallet));

  //generate a new mint signer for adress
  const mint = await generateKeyPairSigner();

  //get the size of the mint
  const space = BigInt(getMintSize());

  // get the minimum balance for rent exemption
  const rent = await rpc.getMinimumBalanceForRentExemption(space).send();

  const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();

  const sendAndConfirm = sendAndConfirmTransactionFactory({
    rpc,
    rpcSubscriptions,
  });

  const msg = createTransactionMessage({ version: 0 });

  const msgWithPayer = setTransactionMessageFeePayerSigner(signer, msg);

  const msgWithLifetime = setTransactionMessageLifetimeUsingBlockhash(
    latestBlockhash,
    msgWithPayer,
  );

  const txMessage = appendTransactionMessageInstructions(
    [
      getCreateAccountInstruction({
        payer: signer,
        newAccount: mint,
        lamports: rent,
        space,
        programAddress: TOKEN_PROGRAM_ADDRESS,
      }),

      getInitializeMintInstruction({
        mint: mint.address,
        decimals: 6,
        mintAuthority: signer.address,
      }),
    ],
    msgWithLifetime,
  );

  const signedTx = await signTransactionMessageWithSigners(txMessage);

  assertIsTransactionWithBlockhashLifetime(signedTx);

  //send and confirm the transaction
  await sendAndConfirm(signedTx, { commitment: "confirmed" });

  const signature = getSignatureFromTransaction(signedTx);

  console.log(
    `mint address: ${mint.address}. Transaction Signature: ${signature}`,
  );

  try {
  } catch (error) {
    console.log(error);
  }
})();
