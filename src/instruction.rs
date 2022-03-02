//! Instruction types
use solana_program::program_error::ProgramError;


use crate::{
    error::TokenError,
};
use std::convert::TryInto;


pub struct ProcessSend {
    pub number: u64,
    pub amounts:Vec<u64>

}
pub struct ProcessClaim {
    pub amount:u64, 

}

pub enum TokenInstruction {
    ProcessSend(ProcessSend),
    ProcessClaim(ProcessClaim),
}

impl TokenInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        use TokenError::InvalidInstruction;
        let (&tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
        Ok(match tag {
            0 => {             
                let (number, rest) = rest.split_at(8);
                let number = number
                    .try_into()
                    .map(u64::from_le_bytes)
                    .or(Err(InvalidInstruction))?; 
                let mut amounts: Vec<u64> = Vec::with_capacity(std::mem::size_of::<u64>()*(number as usize));
                let mut offset=0;
                for _ in 0..number {
                    let (amount,_rest) = rest.split_at(8+offset);
                    let amount=amount
                    .try_into()
                    .map(u64::from_le_bytes)
                    .or(Err(InvalidInstruction))?; 
                    amounts.push(amount);
                    offset=offset+8;
                }               
                Self::ProcessSend(ProcessSend{number,amounts})
            }
            1 => {
                let (amount, _rest) = rest.split_at(8);
                let amount = amount
                    .try_into()
                    .map(u64::from_le_bytes)
                    .or(Err(InvalidInstruction))?;                
                Self::ProcessClaim(ProcessClaim{amount})
            }
            _ => return Err(TokenError::InvalidInstruction.into()),
        })
    }
}
