use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken, token_interface::{Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked, CloseAccount, close_account},
};

use crate::{Escrow, ESCROW_SEED};

// taker
// maker - system
// mint_a
// mint_b
// taker_ata_a
// taker_ata_b
// maker_ata_a
// vault
// escrow
// ata program , token program, system program

#[derive(Accounts)]
pub struct Take<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,
    #[account(mut)]
    pub maker: SystemAccount<'info>,

    #[account(mint::token_program = token_program)]
    pub mint_a: InterfaceAccount<'info, Mint>,

    #[account(mint::token_program = token_program)]
    pub mint_b: InterfaceAccount<'info, Mint>,

    #[account(
        init_if_needed, 
        payer = taker,
        associated_token::mint = mint_a,
        associated_token::authority = taker,
        associated_token::token_program = token_program,
    )]
    pub taker_ata_a: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = taker,
        associated_token::token_program = token_program,
    )]
    pub taker_ata_b: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = mint_b,
        associated_token::authority = maker,
        associated_token::token_program = token_program,
    )]
    pub maker_ata_b: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        close = maker,
        seeds = [ESCROW_SEED, escrow.maker.as_ref() ],
        bump = escrow.bump,
        has_one = mint_a,
        has_one = mint_b,
        has_one = maker,
    )]
    pub escrow: Account<'info, Escrow>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        associated_token::token_program = token_program,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Take<'info>{
    // transfer tokens from taker to maker
    pub fn deposit(&mut self ) -> Result<()>{

        let transfer_acccounts = TransferChecked{
            from: self.taker_ata_b.to_account_info(),
            to: self.maker_ata_b.to_account_info(),
            mint: self.mint_b.to_account_info(),
            authority: self.taker.to_account_info()
        };

        let cpi_ctx = CpiContext::new(self.token_program.key(), transfer_acccounts);
        transfer_checked(cpi_ctx, self.escrow.receive, self.mint_b.decimals)?;
        Ok(())
    }
    // withdraw tokens from taker from the vault and close the escrow
    pub fn withdraw_and_close(&mut self, ) -> Result<()>{

        let transfer_acccounts = TransferChecked{
            from: self.vault.to_account_info(),
            to: self.taker_ata_a.to_account_info(),
            mint: self.mint_a.to_account_info(),
            authority: self.escrow.to_account_info()
        };

        let seeds = [ESCROW_SEED, self.escrow.maker.as_ref(), &[self.escrow.bump]];
        let signer_seeds = [&seeds[..]];


        let cpi_ctx = CpiContext::new_with_signer(self.token_program.key(), transfer_acccounts, &signer_seeds);
        transfer_checked(cpi_ctx, self.escrow.receive, self.mint_a.decimals)?;

        let cpi_accounts = CloseAccount{
            account: self.vault.to_account_info(),
            destination: self.maker.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        let cpi_ctx = CpiContext::new_with_signer(self.token_program.key(), cpi_accounts, &signer_seeds);

        close_account(cpi_ctx)?;
        Ok(())
    }
}