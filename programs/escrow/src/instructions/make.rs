use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked},
};

use crate::{ESCROW_SEED, Escrow};

#[derive(Accounts)]
pub struct Make<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    #[account(mint::token_program = token_program)]
    pub mint_a: InterfaceAccount<'info, Mint>,

    #[account(mint::token_program = token_program)]
    pub mint_b: InterfaceAccount<'info, Mint>,

    #[account(
        mut, 
        associated_token::mint = mint_a,
        associated_token::authority = maker,
        associated_token::token_program = token_program,
    )]
    pub maker_ata_a: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer=maker,
        seeds = [ESCROW_SEED, maker.key().as_ref(), ],
        bump,
        space = Escrow::INIT_SPACE + Escrow::DISCRIMINATOR.len(),
    )]
    pub escrow: Account<'info, Escrow>,

    #[account(
        init,
        payer = maker,        
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        associated_token::token_program = token_program,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}


impl<'info> Make<'info>{

    // initialize the escrow
    pub fn init(&mut self, seed: u64, receive: u64, bumps: &MakeBumps) -> Result<()>{
        self.escrow.set_inner(
            Escrow { 
                seed, 
                maker: self.maker.key(), 
                mint_a: self.mint_a.key(), 
                mint_b: self.mint_b.key(), 
                receive, 
                bump: bumps.escrow,
            }
        );
        Ok(())
    }

    // deposit in the escrow

    pub fn deposit(&mut self, deposit: u64) -> Result<()>{

        let transfer_acccounts = TransferChecked{
            from: self.maker_ata_a.to_account_info(),
            to: self.vault.to_account_info(),
            mint: self.mint_a.to_account_info(),
            authority: self.maker.to_account_info()
        };

        let cpi_ctx = CpiContext::new(self.token_program.key(), transfer_acccounts);
        transfer_checked(cpi_ctx, deposit, self.mint_a.decimals)?;
        Ok(())
    }

}