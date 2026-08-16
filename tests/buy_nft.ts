import * as anchor from "@coral-xyz/anchor";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
} from "@solana/spl-token";
import idl from "../target/idl/nector.json";

// Same platform fee wallet hardcoded in the on-chain buy_nft handler.
const FEE_WALLET = new anchor.web3.PublicKey(
  "GCcZkwkhGhzqBt6Eoc2nJCZFvgYdFAnh1hWuuARi774Z"
);

// Usage:
//   npx ts-node -T tests/buy_nft.ts <SELLER_ADDRESS> <MINT_ADDRESS>
//
// Example:
//   npx ts-node -T tests/buy_nft.ts BbzdZAvBuNDcagzuUrzbwTiaaRkChAu5x8XMUkYbHYDt 7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU

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
  const seller = new anchor.web3.PublicKey(args[0]);
  const mint = new anchor.web3.PublicKey(args[1]);

  if (!seller || !mint) {
    throw new Error("Usage: buy_nft.ts <SELLER_ADDRESS> <MINT_ADDRESS>");
  }

  const buyer = provider.wallet.publicKey;

  // ---------- listing PDA ----------
  const [listingPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("nft_listing"), seller.toBuffer(), mint.toBuffer()],
    program.programId
  );

  // ---------- read the listing so we can show the price before paying ----------
  const listingAccountClient = program.account.nftListing as any;
  const listing = await listingAccountClient.fetch(listingPda);
  const priceLamports = listing.priceLamports as anchor.BN;
  const priceSol = priceLamports.toNumber() / anchor.web3.LAMPORTS_PER_SOL;

  console.log("Listing PDA:", listingPda.toBase58());
  console.log("Price:", priceSol, "SOL");

  // ---------- vault ATA (authority = listing PDA) and buyer's destination ATA ----------
  const vaultNftAta = getAssociatedTokenAddressSync(mint, listingPda, true);
  const buyerNftAta = getAssociatedTokenAddressSync(mint, buyer);

  const tx = await program.methods
    .buyNft()
    .accounts({
      listing: listingPda,
      mint,
      vaultNftAta,
      buyerNftAta,
      buyer,
      seller,
      feeWallet: FEE_WALLET,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .rpc();

  console.log("");
  console.log("----------------------");
  console.log("|   NFT purchased!   |");
  console.log("----------------------");
  console.log("Mint:", mint.toBase58());
  console.log("Seller:", seller.toBase58());
  console.log("Paid:", priceSol, "SOL");
  console.log("----------------------");
  console.log("TX:", tx);
  console.log("");
}

main();
