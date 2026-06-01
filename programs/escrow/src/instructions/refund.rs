use anchor_lang::prelude::*;
use anchor_spl::{token_interface::{
    CloseAccount, Mint, TokenAccount, TokenInterface, TransferChecked, close_account, transfer_checked
}};

use crate::{Escrow, ESCROW_SEED};

// maker - signer
// mint_a
// maker_ata_a
// escrow
// vault
// token_program, system_program
#[derive(Accounts)]
pub struct Refund<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    pub mint_a: InterfaceAccount<'info, Mint>,

    #[account(mut, associated_token::mint = mint_a, associated_token::authority = maker, associated_token::token_program = token_program)]
    pub maker_ata_a: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        close = maker,
        has_one = mint_a,
        has_one = maker,
        seeds = [ESCROW_SEED, maker.key().as_ref()], 
        bump = escrow.bump,
    )]
    pub escrow: Account<'info, Escrow>,

    #[account(mut, associated_token::mint = mint_a, associated_token::authority = escrow, associated_token::token_program = token_program)]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Refund<'info> {
    pub fn refund_and_close_vault(&mut self) -> Result<()> {
        let seeds = [ESCROW_SEED, self.escrow.maker.as_ref(), &[self.escrow.bump]];
        let signer_seeds = [&seeds[..]];
        let transfer_acccounts = TransferChecked {
            from: self.vault.to_account_info(),
            to: self.maker_ata_a.to_account_info(),
            mint: self.mint_a.to_account_info(),
            authority: self.escrow.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(self.token_program.key(),transfer_acccounts , &signer_seeds,);
        transfer_checked(cpi_ctx, self.vault.amount, self.mint_a.decimals)?;

        let cpi_accounts = CloseAccount {
            account: self.vault.to_account_info(),
            destination: self.maker.to_account_info(),
            authority: self.escrow.to_account_info(),
        };
        let cpi_ctx2 = CpiContext::new_with_signer(self.token_program.key(), cpi_accounts, &signer_seeds);
        close_account(cpi_ctx2)?;
        Ok(())
    }
}
