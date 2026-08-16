use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, CloseAccount, Mint, Token, TokenAccount, Transfer},
};
use crate::{ErrorCode, NftListing};

pub fn cancel_nft_listing_handler(ctx: Context<CancelNftListing>) -> Result<()> {
    require!(ctx.accounts.listing.state == 0, ErrorCode::ListingNotActive);
    require!(
        ctx.accounts.seller.key() == ctx.accounts.listing.seller,
        ErrorCode::InvalidListingSeller
    );

    let seller_key = ctx.accounts.listing.seller;
    let mint_key = ctx.accounts.listing.mint;
    let bump = ctx.accounts.listing.bump;
    let signer_seeds: &[&[u8]] = &[
        b"nft_listing",
        seller_key.as_ref(),
        mint_key.as_ref(),
        &[bump],
    ];

    // Return the NFT from the vault back to the seller.
    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault_nft_ata.to_account_info(),
                to: ctx.accounts.seller_nft_ata.to_account_info(),
                authority: ctx.accounts.listing.to_account_info(),
            },
            &[signer_seeds],
        ),
        1,
    )?;

    // Reclaim the vault ATA's rent back to the seller.
    token::close_account(CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        CloseAccount {
            account: ctx.accounts.vault_nft_ata.to_account_info(),
            destination: ctx.accounts.seller.to_account_info(),
            authority: ctx.accounts.listing.to_account_info(),
        },
        &[signer_seeds],
    ))?;

    // `listing` itself is closed by the `close = seller` constraint below.

    Ok(())
}

#[derive(Accounts)]
pub struct CancelNftListing<'info> {
    #[account(
        mut,
        seeds = [b"nft_listing", listing.seller.as_ref(), listing.mint.as_ref()],
        bump = listing.bump,
        close = seller
    )]
    pub listing: Account<'info, NftListing>,

    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = listing,
    )]
    pub vault_nft_ata: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = seller,
        associated_token::mint = mint,
        associated_token::authority = seller,
    )]
    pub seller_nft_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub seller: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
