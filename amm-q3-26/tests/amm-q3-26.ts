import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AmmQ326 } from "../target/types/amm_q3_26";
import {
  createMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
} from "@solana/spl-token";

describe("amm-q3-26", () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  const provider = anchor.getProvider();
  const program = anchor.workspace.ammQ326 as Program<AmmQ326>;
  const wallet = provider.wallet as anchor.Wallet;

  it("initializes the AMM", async () => {
    const mintX = await createMint(
      provider.connection,
      wallet.payer,
      wallet.publicKey,
      null,
      6,
    );

    const mintY = await createMint(
      provider.connection,
      wallet.payer,
      wallet.publicKey,
      null,
      6,
    );

    const seed = new anchor.BN(123);
    const fee = 30;

    const [config] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("config"),
        seed.toArrayLike(Buffer, "le", 8),
      ],
      program.programId,
    );

    const [mintLp] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("lp"), config.toBuffer()],
      program.programId,
    );

    const treasury = wallet.publicKey;

    const vaultX = anchor.utils.token.associatedAddress({
      mint: mintX,
      owner: config,
    });

    const vaultY = anchor.utils.token.associatedAddress({
      mint: mintY,
      owner: config,
    });

    const treasuryX = anchor.utils.token.associatedAddress({
      mint: mintX,
      owner: treasury,
    });

    const treasuryY = anchor.utils.token.associatedAddress({
      mint: mintY,
      owner: treasury,
    });

    await program.methods
      .initialize(seed, fee, wallet.publicKey)
      .accounts({
        initializer: wallet.publicKey,
        mintX,
        mintY,
        mintLp,
        vaultX,
        vaultY,
        treasuryX,
        treasuryY,
        treasury,
        config,
      })
      .rpc();

    console.log("AMM initialized");
    console.log("Config:", config.toString());
    console.log("LP mint:", mintLp.toString());
  });
    it("deposits liquidity", async () => {
    const mintX = await createMint(
      provider.connection,
      wallet.payer,
      wallet.publicKey,
      null,
      6,
    );

    const mintY = await createMint(
      provider.connection,
      wallet.payer,
      wallet.publicKey,
      null,
      6,
    );

    const seed = new anchor.BN(456);

    const [config] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("config"),
        seed.toArrayLike(Buffer, "le", 8),
      ],
      program.programId,
    );

    const [mintLp] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("lp"), config.toBuffer()],
      program.programId,
    );

    const treasury = wallet.publicKey;

    const vaultX = anchor.utils.token.associatedAddress({
      mint: mintX,
      owner: config,
    });

    const vaultY = anchor.utils.token.associatedAddress({
      mint: mintY,
      owner: config,
    });

    const treasuryX = anchor.utils.token.associatedAddress({
      mint: mintX,
      owner: treasury,
    });

    const treasuryY = anchor.utils.token.associatedAddress({
      mint: mintY,
      owner: treasury,
    });

    await program.methods
      .initialize(seed, 30, wallet.publicKey)
      .accounts({
        initializer: wallet.publicKey,
        mintX,
        mintY,
        mintLp,
        vaultX,
        vaultY,
        treasuryX,
        treasuryY,
        treasury,
        config,
      })
      .rpc();

    const userX = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      wallet.payer,
      mintX,
      wallet.publicKey,
    );

    const userY = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      wallet.payer,
      mintY,
      wallet.publicKey,
    );

    await mintTo(
      provider.connection,
      wallet.payer,
      mintX,
      userX.address,
      wallet.publicKey,
      100_000_000,
    );

    await mintTo(
      provider.connection,
      wallet.payer,
      mintY,
      userY.address,
      wallet.publicKey,
      100_000_000,
    );

    const userLp = anchor.utils.token.associatedAddress({
      mint: mintLp,
      owner: wallet.publicKey,
    });

    await program.methods
      .deposit(
        new anchor.BN(100_000_000),
        new anchor.BN(100_000_000),
        new anchor.BN(100_000_000),
      )
      .accounts({
        user: wallet.publicKey,
        mintX,
        mintY,
        config,
        mintLp,
        vaultX,
        vaultY,
        userX: userX.address,
        userY: userY.address,
        userLp,
      })
      .rpc();

    const lpAccount =
      await provider.connection.getTokenAccountBalance(userLp);

    console.log("LP tokens:", lpAccount.value.amount);
  });
  it("withdraws liquidity", async () => {
  const mintX = await createMint(
    provider.connection,
    wallet.payer,
    wallet.publicKey,
    null,
    6,
  );

  const mintY = await createMint(
    provider.connection,
    wallet.payer,
    wallet.publicKey,
    null,
    6,
  );

  const seed = new anchor.BN(789);

  const [config] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("config"), seed.toArrayLike(Buffer, "le", 8)],
    program.programId,
  );

  const [mintLp] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("lp"), config.toBuffer()],
    program.programId,
  );

  const vaultX = anchor.utils.token.associatedAddress({
    mint: mintX,
    owner: config,
  });

  const vaultY = anchor.utils.token.associatedAddress({
    mint: mintY,
    owner: config,
  });

  const treasury = wallet.publicKey;

  const treasuryX = anchor.utils.token.associatedAddress({
    mint: mintX,
    owner: treasury,
  });

  const treasuryY = anchor.utils.token.associatedAddress({
    mint: mintY,
    owner: treasury,
  });

  await program.methods
    .initialize(seed, 30, wallet.publicKey)
    .accounts({
      initializer: wallet.publicKey,
      mintX,
      mintY,
      mintLp,
      vaultX,
      vaultY,
      treasuryX,
      treasuryY,
      treasury,
      config,
    })
    .rpc();

  const userX = await getOrCreateAssociatedTokenAccount(
    provider.connection,
    wallet.payer,
    mintX,
    wallet.publicKey,
  );

  const userY = await getOrCreateAssociatedTokenAccount(
    provider.connection,
    wallet.payer,
    mintY,
    wallet.publicKey,
  );

  await mintTo(
    provider.connection,
    wallet.payer,
    mintX,
    userX.address,
    wallet.publicKey,
    100_000_000,
  );

  await mintTo(
    provider.connection,
    wallet.payer,
    mintY,
    userY.address,
    wallet.publicKey,
    100_000_000,
  );

  const userLp = anchor.utils.token.associatedAddress({
    mint: mintLp,
    owner: wallet.publicKey,
  });

  await program.methods
    .deposit(
      new anchor.BN(100_000_000),
      new anchor.BN(100_000_000),
      new anchor.BN(100_000_000),
    )
    .accounts({
      user: wallet.publicKey,
      mintX,
      mintY,
      config,
      mintLp,
      vaultX,
      vaultY,
      userX: userX.address,
      userY: userY.address,
      userLp,
    })
    .rpc();

  await program.methods
    .withdraw(
      new anchor.BN(50_000_000),
      new anchor.BN(49_000_000),
      new anchor.BN(49_000_000),
    )
    .accounts({
      user: wallet.publicKey,
      mintX,
      mintY,
      config,
      mintLp,
      vaultX,
      vaultY,
      userX: userX.address,
      userY: userY.address,
      userLp,
    })
    .rpc();

  const lpAccount =
    await provider.connection.getTokenAccountBalance(userLp);

  console.log("Remaining LP tokens:", lpAccount.value.amount);
});
it("swaps tokens and collects protocol fee", async () => {
  const mintX = await createMint(
    provider.connection,
    wallet.payer,
    wallet.publicKey,
    null,
    6,
  );

  const mintY = await createMint(
    provider.connection,
    wallet.payer,
    wallet.publicKey,
    null,
    6,
  );

  const seed = new anchor.BN(1000);

  const [config] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("config"), seed.toArrayLike(Buffer, "le", 8)],
    program.programId,
  );

  const [mintLp] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("lp"), config.toBuffer()],
    program.programId,
  );

  const vaultX = anchor.utils.token.associatedAddress({
    mint: mintX,
    owner: config,
  });

  const vaultY = anchor.utils.token.associatedAddress({
    mint: mintY,
    owner: config,
  });

  const treasury = anchor.web3.Keypair.generate().publicKey;

  const treasuryX = anchor.utils.token.associatedAddress({
    mint: mintX,
    owner: treasury,
  });

  const treasuryY = anchor.utils.token.associatedAddress({
    mint: mintY,
    owner: treasury,
  });

  await program.methods
    .initialize(seed, 30, wallet.publicKey)
    .accounts({
      initializer: wallet.publicKey,
      mintX,
      mintY,
      mintLp,
      vaultX,
      vaultY,
      treasuryX,
      treasuryY,
      treasury,
      config,
    })
    .rpc();

  const userX = await getOrCreateAssociatedTokenAccount(
    provider.connection,
    wallet.payer,
    mintX,
    wallet.publicKey,
  );

  const userY = await getOrCreateAssociatedTokenAccount(
    provider.connection,
    wallet.payer,
    mintY,
    wallet.publicKey,
  );

  await mintTo(
    provider.connection,
    wallet.payer,
    mintX,
    userX.address,
    wallet.publicKey,
    200_000_000,
  );

  await mintTo(
    provider.connection,
    wallet.payer,
    mintY,
    userY.address,
    wallet.publicKey,
    200_000_000,
  );

  const userLp = anchor.utils.token.associatedAddress({
    mint: mintLp,
    owner: wallet.publicKey,
  });

  await program.methods
    .deposit(
      new anchor.BN(100_000_000),
      new anchor.BN(100_000_000),
      new anchor.BN(100_000_000),
    )
    .accounts({
      user: wallet.publicKey,
      mintX,
      mintY,
      config,
      mintLp,
      vaultX,
      vaultY,
      userX: userX.address,
      userY: userY.address,
      userLp,
    })
    .rpc();

  const treasuryBefore =
    await provider.connection.getTokenAccountBalance(treasuryX);
    console.log("Treasury X BEFORE:", treasuryBefore.value.amount);

  await program.methods
    .swap(
      true,
      new anchor.BN(10_000_000),
      new anchor.BN(1),
    )
    .accounts({
      user: wallet.publicKey,
      mintX,
      mintY,
      config,
      mintLp,
      vaultX,
      vaultY,
      userX: userX.address,
      userY: userY.address,
      treasuryX,
      treasuryY,
    })
    .rpc();

  const treasuryAfter =
    await provider.connection.getTokenAccountBalance(treasuryX);
    console.log("Treasury X AFTER:", treasuryAfter.value.amount);

  const treasuryIncrease =
  Number(treasuryAfter.value.amount) -
  Number(treasuryBefore.value.amount);

console.log("Treasury X fee:", treasuryIncrease);

if (treasuryIncrease !== 30_000) {
  throw new Error(
    `Expected treasury fee of 30000, got ${treasuryIncrease}`,
  );
}
});
});