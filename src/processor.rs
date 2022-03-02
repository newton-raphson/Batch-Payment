//! Program state processor

use borsh::{BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke,invoke_signed},
    sysvar::{Sysvar,rent::Rent},
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
};

use crate::{
    instruction::{ProcessSend,ProcessClaim,TokenInstruction},
    state::Payments,
};

  pub const PREFIX: &str = "batch";
pub struct Processor {}

impl Processor {
    pub fn process_send(program_id: &Pubkey,accounts: &[AccountInfo],number:u64,amount:Vec<u64>) -> ProgramResult 
    {
        //executed once
        let account_info_iter = &mut accounts.iter();
        let payer_account = next_account_info(account_info_iter)?; // admin who updates the price
        let system_program = next_account_info(account_info_iter)?;
        let vault =next_account_info(account_info_iter)?;
        let pda_data =next_account_info(account_info_iter)?; //account to save data // this account gives the price feed
     
        //Was the transaction signed by admin account's private key
        if !payer_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        let (vault_address, _bump_seed) = Pubkey::find_program_address(
            &[
                &payer_account.key.to_bytes(),
                PREFIX.as_bytes(),
            ],
            program_id,
        );
        if vault_address!=*vault.key
        {
            return Err(ProgramError::MissingRequiredSignature);
        }
        msg!("The instruction is signed");        
          //Was the transaction updated by admin account
        let rent = Rent::get()?;
        let size: u64=std::mem::size_of::<Payments>() as u64 + number*(std::mem::size_of::<Pubkey>()+std::mem::size_of::<Pubkey>()) as u64;
        let transfer_amount =  rent.minimum_balance (size as usize);
       //creating the data feed account
       msg!("The payment data account is created...");
        invoke(
            &system_instruction::create_account(
                payer_account.key,
                pda_data.key,
                transfer_amount,
                size,
                program_id,
            ),
            &[
                payer_account.clone(),
                pda_data.clone(),
                system_program.clone(),
            ],
        )?;
        msg!("The payment account is complete being created");
        let mut pda_start = Payments::from_account(pda_data)?;
        msg!("Data writing...");
        //escrow.signed_by.push(signed_by);
        let mut sending_amount: u64=0;
        let mut i:usize=0;
        while i<(number as usize)
        {
            let payeeee = next_account_info(account_info_iter)?;
            pda_start.payee.push(*payeeee.key);
            sending_amount+=amount[i];
            i=i+1;
        }
        invoke(
            &system_instruction::transfer(
                payer_account.key,
                vault.key,
                sending_amount,
            ),
            &[
                payer_account.clone(),
                vault.clone(),
                system_program.clone()
            ],
        )?;
        pda_start.payer=*payer_account.key;
        pda_start.amounts=amount;
        pda_start.serialize(&mut *pda_data.data.borrow_mut())?;
        msg!("Data writing complete");
        Ok(())
    }
    
    pub fn process_claim(program_id: &Pubkey,accounts: &[AccountInfo],_amount:u64)->ProgramResult
    {  
        //changing the price

        let account_info_iter = &mut accounts.iter();
        let payee_account =next_account_info(account_info_iter)?;
        let payer_account = next_account_info(account_info_iter)?; 
        let pda_data =next_account_info(account_info_iter)?; 
        let vault=next_account_info(account_info_iter)?; 
        let system_program=next_account_info(account_info_iter)?;
        msg!("Verifying ...");
        if !payer_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        let (vault_address, bump_seed) = Pubkey::find_program_address(
            &[
                &payer_account.key.to_bytes(),
                PREFIX.as_bytes(),
            ],
            program_id,
        );
        let pda_signer_seeds: &[&[_]] = &[
            &payer_account.key.to_bytes(),
            PREFIX.as_bytes(),
            &[bump_seed],
        ];
        if pda_data.owner != program_id
        {
            return Err(ProgramError::MissingRequiredSignature);
        } 

        if vault_address!=*vault.key
        {
            return Err(ProgramError::MissingRequiredSignature);
        }
        let mut pda_start = Payments::from_account(pda_data)?;

        if *payer_account.key !=pda_start.payer
        {
            return Err(ProgramError::MissingRequiredSignature);
        }
        let mut transfer_amount=0;

        for i in 0..pda_start.payee.len()
        {
           
            if *payee_account.key == pda_start.payee[i]
            {
                transfer_amount=pda_start.amounts[i];
                pda_start.amounts[i]=0;

            }
        }
        if transfer_amount>0
        {
            invoke_signed(
                &system_instruction::transfer(
                    vault.key,
                    payee_account.key,
                    transfer_amount,
                ),
                &[
                    payee_account.clone(),
                    vault.clone(),
                    system_program.clone()
                ],&[&pda_signer_seeds],
            )?;
        }
        else
        {
            msg!("Your Account is not valid");
            return Err(ProgramError::MissingRequiredSignature);
        }
      
        pda_start.serialize(&mut *pda_data.data.borrow_mut())?;
        msg!("Successfully Done");
        Ok(())

    }
        
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
        let instruction = TokenInstruction::unpack(input)?;
        match instruction {
            TokenInstruction::ProcessSend(ProcessSend{number,amounts}) => {
                msg!("Instruction: Sending");
                Self::process_send(program_id, accounts,number,amounts)
            }
            TokenInstruction::ProcessClaim(ProcessClaim{ amount }) => {
                msg!("Instruction: Claim");
                Self::process_claim(program_id, accounts, amount)
            }
        }
    }
}
