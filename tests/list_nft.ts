import * as anchor from "@coral-xyz/anchor";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
} from "@solana/spl-token";
import idl from "../target/idl/nector.json";

// Usage:
//   npx ts-node -T tests/list_nft.ts <MINT_ADDRESS> <PRICE_SOL>
//
// Example:
//   npx ts-node -T tests/list_nft.ts 7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU 1.5

async function main() {
  const connection = new anchor.web3.Connection(
    "https://api.devnet.solana.com",
    "confirmed"
  );

  const wallet = anchor.Wallet.local();
  const provider = new anchor.AnchorProvider(connection, wallet, {
    commitment: "confirmed",
  });
  anchor.setProvider(provider);

  const program = new anchor.Program(idl as any, provider);

  const args = process.argv.slice(2);
  const mint = new anchor.web3.PublicKey(args[0]);
  const priceSol = parseFloat(args[1]);

  if (!mint || Number.isNaN(priceSol)) {
    throw new Error(
      "Usage: list_nft.ts <MINT_ADDRESS> <PRICE_SOL>"
    );
  }

  const priceLamports = new anchor.BN(
    Math.round(priceSol * anchor.web3.LAMPORTS_PER_SOL)
  );

  const seller = provider.wallet.publicKey;

  // ---------- listing PDA ----------
  const [listingPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("nft_listing"), seller.toBuffer(), mint.toBuffer()],
    program.programId
  );

  // ---------- seller's existing ATA holding the NFT ----------
  const sellerNftAta = getAssociatedTokenAddressSync(mint, seller);

  // ---------- program-controlled vault ATA (authority = listing PDA) ----------
  const vaultNftAta = getAssociatedTokenAddressSync(mint, listingPda, true);

  console.log("Listing PDA:", listingPda.toBase58());
  console.log("Vault ATA:", vaultNftAta.toBase58());

  const tx = await program.methods
    .listNft(priceLamports)
    .accounts({
      listing: listingPda,
      mint,
      sellerNftAta,
      vaultNftAta,
      seller,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .rpc();

  console.log("");
  console.log("----------------------");
  console.log("|   NFT listed!      |");
  console.log("----------------------");
  console.log("Mint:", mint.toBase58());
  console.log("Price:", priceSol, "SOL");
  console.log("----------------------");
  console.log("TX:", tx);
  console.log("Listing PDA:", listingPda.toBase58());
  console.log("");
}

main();
