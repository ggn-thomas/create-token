use borsh::{BorshDeserialize, BorshSerialize};
use mpl_token_metadata::{
    instructions::{CreateMetadataAccountV3,CreateMetadataAccountV3InstructionArgs},
    types::DataV2,
};

use spl_associated_token_account::instruction as associated_token_account_instruction;

use spl_token::instruction as token_instruction;
use spl_token::state::Mint;

#[allow(deprecated)]
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    pubkey::Pubkey,
    sysvar::{rent::Rent, Sysvar},
    system_instruction,
    program_pack::Pack,
};

// data struct
#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct CreateTokenArgs {
    pub token_title: String,
    pub token_symbol: String,
    pub token_uri: String,
    pub token_decimals: u8,
    pub initial_supply: u64,
}

entrypoint!(process_instruction); // handles the deserialization of data in the parameters of the process_instruction function

fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {

    let args = CreateTokenArgs::try_from_slice(instruction_data)?;

    // returns an index for the first account
    //
    let accounts_iter = &mut accounts.iter(); // reference to the iterator, mut = you can modify the ref to it

    let mint_account = next_account_info(accounts_iter)?;
    let mint_authority = next_account_info(accounts_iter)?;
    let associated_token_account = next_account_info(accounts_iter)?;
    let metadata_account = next_account_info(accounts_iter)?;
    let payer = next_account_info(accounts_iter)?;
    let rent = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;
    let token_metadata_program = next_account_info(accounts_iter)?;
    let associated_token_program = next_account_info(accounts_iter)?;

    // create mint account
    //
    msg!("Creating mint account...");

    invoke(
        &system_instruction::create_account(
            payer.key, 
            mint_account.key,
            Rent::get()?.minimum_balance(Mint::LEN),
            Mint::LEN as u64,
            token_program.key,
        ),
        &[
            mint_account.clone(),
            payer.clone(),
            system_program.clone(),
            token_program.clone(),
        ],
    )?;

    // init mint account
    //
    msg!("Initializing mint...");
    invoke(
        &token_instruction::initialize_mint(
            token_program.key,
            mint_account.key,
            mint_authority.key,
            Some(mint_authority.key),
            args.token_decimals,
        )?,
        &[
            mint_account.clone(),
            mint_authority.clone(),
            token_program.clone(),
            rent.clone(),
        ],
    )?;

    // create metadata account
    //
    msg!("Creating metadata account...");
    invoke(
        &CreateMetadataAccountV3 {
            metadata: *metadata_account.key,
            mint: *mint_account.key,
            mint_authority: *mint_authority.key,
            payer: *payer.key,
            update_authority: (*mint_authority.key, true),
            system_program: *system_program.key,
            rent: None,
        }
        .instruction(CreateMetadataAccountV3InstructionArgs {
            data: DataV2 {
                name: args.token_title,
                symbol: args.token_symbol,
                uri: args.token_uri,
                seller_fee_basis_points: 0,
                creators: None,
                collection: None,
                uses: None,
            },
            is_mutable: true,
            collection_details: None,
        }),
        &[
            metadata_account.clone(),
            mint_account.clone(),
            mint_authority.clone(),
            payer.clone(),
            system_program.clone(),
            token_metadata_program.clone(),
        ],
    )?;

    // create supply account
    //
    msg!("Creating associated token account...");
    invoke(
        &associated_token_account_instruction::create_associated_token_account(
            payer.key,
            payer.key,
            mint_account.key,
            token_program.key,
        ),
        &[
            mint_account.clone(),
            associated_token_account.clone(),
            payer.clone(),
            system_program.clone(),
            token_program.clone(),
            associated_token_program.clone(),
        ],
    )?;
    // Miné un token vers un compte token associé
    //
    msg!("Minning token to associated token account...");
    invoke(
        &token_instruction::mint_to(
            token_program.key,
            mint_account.key,
            associated_token_account.key,
            mint_authority.key,
            &[mint_authority.key],
            args.initial_supply,
        )?,
        &[
            mint_account.clone(),
            mint_authority.clone(),
            associated_token_account.clone(),
            token_program.clone(),
        ],
    )?;

    Ok(())
}