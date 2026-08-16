import * as anchor from "@coral-xyz/anchor";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
} from "@solana/spl-token";
import idl from "../target/idl/nector.json";

// Usage:
//   npx ts-node -T tests/cancel_nft_listing.ts <MINT_ADDRESS>
//
// Example:
//   npx ts-node -T tests/cancel_nft_listing.ts 7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU

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

  if (!mint) {
    throw new Error("Usage: cancel_nft_listing.ts <MINT_ADDRESS>");
  }

  const seller = provider.wallet.publicKey;

  // ---------- listing PDA ----------
  const [listingPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("nft_listing"), seller.toBuffer(), mint.toBuffer()],
    program.programId
  );

  // ---------- vault ATA (authority = listing PDA) and seller's return ATA ----------
  const vaultNftAta = getAssociatedTokenAddressSync(mint, listingPda, true);
  const sellerNftAta = getAssociatedTokenAddressSync(mint, seller);

  console.log("Listing PDA:", listingPda.toBase58());

  const tx = await program.methods
    .cancelNftListing()
    .accounts({
      listing: listingPda,
      mint,
      vaultNftAta,
      sellerNftAta,
      seller,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .rpc();

  console.log("");
  console.log("----------------------");
  console.log("| Listing cancelled! |");
  console.log("----------------------");
  console.log("Mint:", mint.toBase58());
  console.log("----------------------");
  console.log("TX:", tx);
  console.log("");
}

main();
