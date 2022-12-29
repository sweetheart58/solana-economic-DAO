use solana_program::{
    account_info::{AccountInfo, next_account_info},
    entrypoint,
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    system_instruction,
    msg,
};
use spl_token;
use std::convert::TryInto;


entrypoint!(process_instruction);


// on-chain program instruction function
// functions arguments are just the Solana boilerplate ones
pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    input: &[u8],
) -> ProgramResult {

    // read accounts
    let acc_iter = &mut accounts.iter();
    // 1. Token account we hold
    let pda = next_account_info(acc_iter)?; // program-derived-account owning gigi
    // 2. Token account to send ECOV to
    let user = next_account_info(acc_iter)?; // ecoverse user requesting ECOV
    // 3. Our wallet address
    let ecov_pool = next_account_info(acc_iter)?; // gigi = wallet owning ECOV 
    // 4. Token Program
    let token_program = next_account_info(acc_iter)?; // Solana token program
    // 5. SOL liquidity Cache
    let bbox_sol_payee = next_account_info(acc_iter)?; // gerry = BBox cash-in account


    // deserialized byte array (8 bytes) into an integer
    let amount = input
        .get(..8)
        .and_then(|slice| slice.try_into().ok()) //lambda: turn slice to int
        .map(u64::from_le_bytes)
        .ok_or(ProgramError::InvalidInstructionData)?;

    msg!("Request to recieve {:?} ECOV from user {:?}",
    amount, user.key);
    msg!("SOL Transfer in progress...");

    // Cross program invocations
    // SOL transfer from USER to PAYEE
    invoke(
        &system_instruction::transfer(user.key, bbox_sol_payee.key, amount),
        &[user.clone(), bbox_sol_payee.clone()],
    )?;
    msg!("SOL transfer succeeded!");


    // Find a Program Derived Account (PDA) and call it escrow
    // Deterministically derive the escrow pubkey
    // let (escrow_pubkey, escrow_bump_seed) = Pubkey::find_program_address(&[&["BalloonBox", "-", "escrow"]], &ecov_program);
    // To reduce the compute cost, use find_program_address() fn 
    // off-chain and pass the resulting bump seed to the program.
    // PDA addresses are indistinguishable from any other pubkey.
    // The only way for the runtime to verify that the address belongs to a 
    // program is for the program to supply the seeds used to generate the address.


    // TO DO: add "if SOL transfer is successful, then transfer ECOV, else raise err"
    // TO DO: multiply amount by 10E-9, b/c the standard unit for SOL is Lamports,
    // whereas the stanrard unit for ECOV is the mere decimal system
    // ECOV transfer from ECOV POOL to USER
    msg!("ECOV Transfer in progress...");
    invoke_signed(
        &spl_token::instruction::transfer(
            token_program.key,
            ecov_pool.key,
            user.key,
            pda.key,
            &[],
            amount
        )?,
        &[ecov_pool.clone(), user.clone(), pda.clone()],
        &[
            &[b"BalloonBox-", b"escrow"] // TO DO: enter seeds here!
        ]
    )?;
    msg!("ECOV transfer succeeded!");

    // finally
    Ok(())
}