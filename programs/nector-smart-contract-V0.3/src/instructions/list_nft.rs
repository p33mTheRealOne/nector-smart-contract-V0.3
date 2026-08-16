use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount, Transfer},
};
use crate::{ErrorCode, NftListing};

pub fn list_nft_handler(ctx: Context<ListNft>, price_lamports: u64) -> Result<()> {
    // Basic sanity check that this mint actually looks like an NFT:
    // 0 decimals + a total supply of exactly 1.
    require!(
        ctx.accounts.mint.decimals == 0 && ctx.accounts.mint.supply == 1,
        ErrorCode::NotAnNft
    );

    require!(ctx.accounts.seller_nft_ata.amount == 1, ErrorCode::NotAnNft);

    // Move the NFT out of the seller's wallet into the program-controlled
    // vault ATA. The vault's authority is the `listing` PDA itself, so only
    // this program (via buy_nft / cancel_nft_listing) can move it again.
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.seller_nft_ata.to_account_info(),
                to: ctx.accounts.vault_nft_ata.to_account_info(),
                authority: ctx.accounts.seller.to_account_info(),
            },
        ),
        1,
    )?;

    let listing = &mut ctx.accounts.listing;
    listing.seller = ctx.accounts.seller.key();
    listing.mint = ctx.accounts.mint.key();
    listing.price_lamports = price_lamports;
    listing.state = 0; // Listed
    listing.bump = ctx.bumps.listing;

    Ok(())
}

#[derive(Accounts)]
pub struct ListNft<'info> {
    #[account(
        init,
        payer = seller,
        space = 8 + NftListing::SIZE,
        seeds = [b"nft_listing", seller.key().as_ref(), mint.key().as_ref()],
        bump
    )]
    pub listing: Account<'info, NftListing>,

    /// The NFT mint being listed. Not modified here — only read (decimals,
    /// supply) to sanity-check it's a single-supply NFT.
    pub mint: Account<'info, Mint>,

    /// Seller's existing associated token account holding the NFT.
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = seller,
    )]
    pub seller_nft_ata: Account<'info, TokenAccount>,

    /// Program-controlled vault ATA, created here. Its authority is the
    /// `listing` PDA, so only this program can move the NFT out again.
    #[account(
        init,
        payer = seller,
        associated_token::mint = mint,
        associated_token::authority = listing,
    )]
    pub vault_nft_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub seller: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
