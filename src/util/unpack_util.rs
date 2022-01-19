use crate::{
    error::LendingError,
    math::{Decimal},
};
use solana_program::{
    msg,
    program_error::ProgramError,
    pubkey::{Pubkey, PUBKEY_BYTES},
};
use std::{convert::TryInto};
pub fn unpack_bool(input: &[u8]) -> Result<(bool,&[u8]), ProgramError> {
    let (value, _input) = unpack_u8(input)?;
    let result = match value {
        0 => false,
        1 => true,
        _ => {
            msg!("Boolean cannot be unpacked");
            return Err(ProgramError::InvalidAccountData)
        }
    };
    Ok((result,_input))
}
pub fn unpack_decimal(input: &[u8]) -> Result<(Decimal,&[u8]), ProgramError>{
    if input.len() < 16 {
        msg!("Decimal cannot be unpacked");
        return Err(LendingError::InstructionUnpackError.into());
    }
    let (bytes, rest) = input.split_at(16);
    let src = bytes.try_into().map_err(|_| LendingError::InstructionUnpackError)?;
    Ok((
        Decimal::from_scaled_val(u128::from_le_bytes(src)),
        rest
    ))
}

pub fn unpack_u64(input: &[u8]) -> Result<(u64, &[u8]), ProgramError> {
    if input.len() < 8 {
        msg!("u64 cannot be unpacked");
        return Err(LendingError::InstructionUnpackError.into());
    }
    let (bytes, rest) = input.split_at(8);
    let value = bytes
        .get(..8)
        .and_then(|slice| slice.try_into().ok())
        .map(u64::from_le_bytes)
        .ok_or(LendingError::InstructionUnpackError)?;
    Ok((value, rest))
}

pub fn unpack_u8(input: &[u8]) -> Result<(u8, &[u8]), ProgramError> {
    if input.is_empty() {
        msg!("u8 cannot be unpacked");
        return Err(LendingError::InstructionUnpackError.into());
    }
    let (bytes, rest) = input.split_at(1);
    let value = bytes
        .get(..1)
        .and_then(|slice| slice.try_into().ok())
        .map(u8::from_le_bytes)
        .ok_or(LendingError::InstructionUnpackError)?;
    Ok((value, rest))
}

pub fn unpack_bytes32(input: &[u8]) -> Result<(&[u8; 32], &[u8]), ProgramError> {
    if input.len() < 32 {
        msg!("32 bytes cannot be unpacked");
        return Err(LendingError::InstructionUnpackError.into());
    }
    let (bytes, rest) = input.split_at(32);
    Ok((
        bytes
            .try_into()
            .map_err(|_| LendingError::InstructionUnpackError)?,
        rest,
    ))
}

pub fn unpack_pubkey(input: &[u8]) -> Result<(Pubkey, &[u8]), ProgramError> {
    if input.len() < PUBKEY_BYTES {
        msg!("Pubkey cannot be unpacked");
        return Err(LendingError::InstructionUnpackError.into());
    }
    let (key, rest) = input.split_at(PUBKEY_BYTES);
    let pk = Pubkey::new(key);
    Ok((pk, rest))
}