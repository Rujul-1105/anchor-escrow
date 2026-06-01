use anchor_lang::solana_program::program_pack::Pack;
#[cfg(test)]
use {
    anchor_lang::{
        prelude::*, solana_program::instruction::Instruction,
        system_program::ID as SYSTEM_PROGRAM_ID, AccountDeserialize, InstructionData,
        ToAccountMetas,
    },
    anchor_spl::{
        associated_token::{self, ID as ASSOCIATED_TOKEN_PROGRAM_ID},
        token::{spl_token, ID as TOKEN_PROGRAM_ID},
    },
    litesvm::LiteSVM,
    litesvm_token::{CreateAssociatedTokenAccount, CreateMint, MintTo},
    solana_keypair::Keypair,
    solana_keypair::Signer,
    solana_message::Message,
    solana_transaction::Transaction,
};

fn setup() -> (LiteSVM, Keypair) {
    let program_id = escrow::id();
    let keypair = Keypair::new();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../target/deploy/escrow.so");
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&keypair.pubkey(), 5_000_000_000).unwrap();
    (svm, keypair)
}

#[test]
fn test_make_and_refund() {
    let (mut svm, keypair) = setup();

    let maker = keypair.pubkey();

    let mint_a = CreateMint::new(&mut svm, &keypair)
        .decimals(6)
        .authority(&maker)
        .send()
        .unwrap();
    msg!("Mint A: {}", mint_a);

    let mint_b = CreateMint::new(&mut svm, &keypair)
        .decimals(6)
        .authority(&maker)
        .send()
        .unwrap();
    msg!("Mint B: {}", mint_b);

    let maker_ata_a = CreateAssociatedTokenAccount::new(&mut svm, &keypair, &mint_a)
        .owner(&maker)
        .send()
        .unwrap();
    msg!("Maker ATA A: {}", maker_ata_a);

    let seed = 123u64;
    let escrow = Pubkey::find_program_address(
        &[b"escrow", maker.as_ref(), &seed.to_le_bytes()],
        &escrow::id(),
    );
    msg!("Escrow: {}", escrow.0);
    msg!("Escrow Bump: {}", escrow.1);

    let vault = associated_token::get_associated_token_address(&escrow.0, &mint_a);
    msg!("Vault PDA: {}", vault);

    MintTo::new(&mut svm, &keypair, &mint_a, &maker_ata_a, 100_000_000)
        .send()
        .unwrap();

    let make_ix = Instruction {
        program_id: escrow::id(),
        accounts: escrow::accounts::Make {
            maker,
            mint_a,
            mint_b,
            maker_ata_a,
            vault,
            escrow: escrow.0,
            system_program: SYSTEM_PROGRAM_ID,
            token_program: TOKEN_PROGRAM_ID,
            associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: escrow::instruction::Make {
            deposit: 100_000_000,
            receive: 50_000_000,
            seed: 123u64,
        }
        .data(),
    };

    let message = Message::new(&[make_ix], Some(&keypair.pubkey()));
    let recent_blockhash = svm.latest_blockhash();

    let tx = Transaction::new(&[&keypair], message, recent_blockhash);
    let tx_meta_data = svm.send_transaction(tx).unwrap();

    msg!("Make instruction sent");
    msg!("CUs consumed: {}", tx_meta_data.compute_units_consumed);
    msg!("Txn Signature: {:?}", tx_meta_data.signature);

    let vault_account = svm.get_account(&vault).unwrap();
    let vault_data = spl_token::state::Account::unpack(&vault_account.data).unwrap();
    assert_eq!(vault_data.amount, 100_000_000, "Vault amount is incorrect");
    assert_eq!(vault_data.owner, escrow.0);
    assert_eq!(vault_data.mint, mint_a);

    let escrow_account = svm.get_account(&escrow.0).unwrap();
    let escrow_data =
        escrow::state::Escrow::try_deserialize(&mut escrow_account.data.as_ref()).unwrap();
    assert_eq!(escrow_data.seed, seed);
    assert_eq!(escrow_data.maker, maker);
    assert_eq!(escrow_data.mint_a, mint_a);
    assert_eq!(escrow_data.mint_b, mint_b);
    assert_eq!(escrow_data.receive, 50_000_000);

    let refund_ix = Instruction {
        program_id: escrow::id(),
        accounts: escrow::accounts::Refund {
            maker,
            maker_ata_a,
            mint_a,
            vault,
            escrow: escrow.0,
            system_program: SYSTEM_PROGRAM_ID,
            token_program: TOKEN_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: escrow::instruction::Refund { seed: 123u64 }.data(),
    };

    let message = Message::new(&[refund_ix], Some(&keypair.pubkey()));
    let recent_blockhash = svm.latest_blockhash();
    let tx = Transaction::new(&[&keypair], message, recent_blockhash);
    let tx_meta_data = svm.send_transaction(tx).unwrap();

    msg!("Refund instruction sent");
    msg!("CUs consumed: {}", tx_meta_data.compute_units_consumed);
    msg!("Txn Signature: {:?}", tx_meta_data.signature);

    assert!(svm.get_account(&escrow.0).is_none());
    assert!(svm.get_account(&vault).is_none());
}
