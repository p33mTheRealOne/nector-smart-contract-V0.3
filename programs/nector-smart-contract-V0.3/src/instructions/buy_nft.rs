use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer as SolTransfer};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, CloseAccount, Mint, Token, TokenAccount, Transfer as SplTransfer},
};
use crate::{ErrorCode, NftListing};

pub fn buy_nft_handler(ctx: Context<BuyNft>) -> Result<()> {
    let price = ctx.accounts.listing.price_lamports;

    require!(ctx.accounts.listing.state == 0, ErrorCode::ListingNotActive);
    require!(
        ctx.accounts.seller.key() == ctx.accounts.listing.seller,
        ErrorCode::InvalidListingSeller
    );

    // Same 1% platform fee convention already used everywhere else in this
    // program (buyer_fund_escrow / seller_fund_escrow).
    let fee = price
        .checked_mul(1)
        .ok_or(ErrorCode::MathOverflow)?
        .checked_div(100)
        .ok_or(ErrorCode::MathOverflow)?;
    let seller_receive = price.checked_sub(fee).ok_or(ErrorCode::MathOverflow)?;

    // ---------------- SOL leg: buyer -> seller, buyer -> fee_wallet ----------------

    transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            SolTransfer {
                from: ctx.accounts.buyer.to_account_info(),
                to: ctx.accounts.seller.to_account_info(),
            },
        ),
        seller_receive,
    )?;

    transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            SolTransfer {
                from: ctx.accounts.buyer.to_account_info(),
                to: ctx.accounts.fee_wallet.to_account_info(),
            },
        ),
        fee,
    )?;

    // ---------------- NFT leg: vault -> buyer, signed by the listing PDA ----------------

    let seller_key = ctx.accounts.listing.seller;
    let mint_key = ctx.accounts.listing.mint;
    let bump = ctx.accounts.listing.bump;
    let signer_seeds: &[&[u8]] = &[
        b"nft_listing",
        seller_key.as_ref(),
        mint_key.as_ref(),
        &[bump],
    ];

    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            SplTransfer {
                from: ctx.accounts.vault_nft_ata.to_account_info(),
                to: ctx.accounts.buyer_nft_ata.to_account_info(),
                authority: ctx.accounts.listing.to_account_info(),
            },
            &[signer_seeds],
        ),
        1,
    )?;

    // Vault ATA is empty now — reclaim its rent back to the seller (who paid
    // for it in list_nft) instead of leaving it dangling.
    token::close_account(CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        CloseAccount {
            account: ctx.accounts.vault_nft_ata.to_account_info(),
            destination: ctx.accounts.seller.to_account_info(),
            authority: ctx.accounts.listing.to_account_info(),
        },
        &[signer_seeds],
    ))?;

    // `listing` is closed by the `close = seller` constraint below once this
    // handler returns, which is itself the permanent "sold" marker (the
    // account — and any chance of buying it twice — ceases to exist).

    Ok(())
}

#[derive(Accounts)]
pub struct BuyNft<'info> {
    #[account(
        mut,
        seeds = [b"nft_listing", listing.seller.as_ref(), listing.mint.as_ref()],
        bump = listing.bump,
        close = seller
    )]
    pub listing: Account<'info, NftListing>,

    /// The NFT mint being purchased. Not modified — only used to derive/
    /// validate the vault and buyer associated token accounts.
    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = listing,
    )]
    pub vault_nft_ata: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = buyer,
        associated_token::mint = mint,
        associated_token::authority = buyer,
    )]
    pub buyer_nft_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub buyer: Signer<'info>,

    /// CHECK: verified against listing.seller above; only ever receives
    /// lamports here, never read as data.
    #[account(mut)]
    pub seller: UncheckedAccount<'info>,

    #[account(
        mut,
        address = pubkey!("GCcZkwkhGhzqBt6Eoc2nJCZFvgYdFAnh1hWuuARi774Z")// same platform fee wallet used elsewhere in this program
    )]
    /// CHECK: platform fee wallet
    pub fee_wallet: SystemAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
