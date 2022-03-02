///into state.rs
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info:: AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
    borsh::try_from_slice_unchecked,}; 
//price state
#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct Payments {
     
    pub payee: Vec<Pubkey>,
    pub amounts: Vec<u64>,
    pub payer: Pubkey,
    pub update_time: u64,
    pub price:u64,
}

impl Payments {
    pub fn from_account(account:&AccountInfo)-> Result<Payments, ProgramError> {
            let md: Payments =try_from_slice_unchecked(&account.data.borrow_mut())?;
            Ok(md)
    }
}
