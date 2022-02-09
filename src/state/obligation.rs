use super::*;
use crate::{
    math::{Decimal},
};
use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use solana_program::{
    clock::Slot,
    msg,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::{Pubkey, PUBKEY_BYTES},
};
use std::{
    convert::{TryFrom},
};
use crate::state::last_update::LastUpdate;


/// Max number of collateral and liquidity reserve accounts combined for an obligation
pub const MAX_OBLIGATION_RESERVES: usize = 10;
/// Allow number of collateral and liquidity reserve accounts combined for an obligation
pub const ALLOWED_OBLIGATION_RESERVES: usize = 10;

/// Lending market obligation state
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Obligation {
    /// Version of the struct
    pub version: u8,
    /// Last update to collateral, liquidity, or their market values
    pub last_update: LastUpdate,
    /// Lending market address
    pub lending_market: Pubkey,
    /// Owner authority which can borrow liquidity
    pub owner: Pubkey,
    /// Deposited collateral for the obligation, unique by deposit reserve address
    pub deposits: Vec<ObligationCollateral>,
    /// Borrowed liquidity for the obligation, unique by borrow reserve address
    pub borrows: Vec<ObligationLiquidity>,
    /// Market value of deposits
    pub deposited_value: Decimal,
    /// Market value of borrows
    pub borrowed_value: Decimal,
    /// The maximum borrow value at the weighted average loan to value ratio
    pub allowed_borrow_value: Decimal,
    /// The dangerous borrow value at the weighted average liquidation threshold
    pub unhealthy_borrow_value: Decimal,
    /// Total unclaimed mine for the  in context
    pub unclaimed_mine: Decimal,
}

impl Obligation {
    /// Create a new obligation
    pub fn new(params: InitObligationParams) -> Self {
        let mut obligation = Self::default();
        Self::init(&mut obligation, params);
        obligation
    }

    /// Initialize an obligation
    pub fn init(&mut self, params: InitObligationParams) {
        self.version = PROGRAM_VERSION;
        self.last_update = LastUpdate::new(params.current_slot);
        self.lending_market = params.lending_market;
        self.owner = params.owner;
        self.deposits = params.deposits;
        self.borrows = params.borrows;
    }
}

/// Initialize an obligation
pub struct InitObligationParams {
    /// Last update to collateral, liquidity, or their market values
    pub current_slot: Slot,
    /// Lending market address
    pub lending_market: Pubkey,
    /// Owner authority which can borrow liquidity
    pub owner: Pubkey,
    /// Deposited collateral for the obligation, unique by deposit reserve address
    pub deposits: Vec<ObligationCollateral>,
    /// Borrowed liquidity for the obligation, unique by borrow reserve address
    pub borrows: Vec<ObligationLiquidity>,
}

impl Sealed for Obligation {}
impl IsInitialized for Obligation {
    fn is_initialized(&self) -> bool {
        self.version != UNINITIALIZED_VERSION
    }
}

/// Obligation collateral state
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ObligationCollateral {

    pub index:Decimal,
    /// Reserve collateral is deposited to
    pub deposit_reserve: Pubkey,
    /// Amount of collateral deposited
    pub deposited_amount: u64,
    /// Collateral market value in quote currency
    pub market_value: Decimal,
}

/// Obligation liquidity state
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ObligationLiquidity {

    pub index:Decimal,
    /// Reserve liquidity is borrowed from
    pub borrow_reserve: Pubkey,
    /// Borrow rate used for calculating interest
    pub cumulative_borrow_rate_wads: Decimal,
    /// Amount of liquidity borrowed plus interest
    pub borrowed_amount_wads: Decimal,
    /// Liquidity market value in quote currency
    pub market_value: Decimal,
}


const OBLIGATION_COLLATERAL_LEN: usize = 72; // 32 + 8 + 16 + 16
const OBLIGATION_LIQUIDITY_LEN: usize = 96; // 32 + 16 + 16 + 16
const OBLIGATION_LEN: usize = 1092;// 916; // 1 + 8 + 1 + 32 + 32 + 16 + 16 + 16 + 16 + 1 + 1 + (56 * 1) + (80 * 9)
                                   // @TODO: break this up by obligation / collateral / liquidity https://git.io/JOCca
impl Pack for Obligation {
    const LEN: usize = OBLIGATION_LEN;

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let output = array_mut_ref![dst, 0, OBLIGATION_LEN];
        #[allow(clippy::ptr_offset_with_cast)]
        let (
            version,
            last_update_slot,
            last_update_stale,
            lending_market,
            owner,
            deposited_value,
            borrowed_value,
            allowed_borrow_value,
            unhealthy_borrow_value,
            deposits_len,
            borrows_len,
            unclaimed_mine,
            data_flat,
        ) = mut_array_refs![
            output,
            1,
            8,
            1,
            PUBKEY_BYTES,
            PUBKEY_BYTES,
            16,
            16,
            16,
            16,
            1,
            1,
            16,
            OBLIGATION_COLLATERAL_LEN + (OBLIGATION_LIQUIDITY_LEN * (MAX_OBLIGATION_RESERVES - 1))
        ];

        // obligation
        *version = self.version.to_le_bytes();
        *last_update_slot = self.last_update.slot.to_le_bytes();
        pack_bool(self.last_update.stale, last_update_stale);
        lending_market.copy_from_slice(self.lending_market.as_ref());
        owner.copy_from_slice(self.owner.as_ref());
        pack_decimal(self.deposited_value, deposited_value);
        pack_decimal(self.borrowed_value, borrowed_value);
        pack_decimal(self.allowed_borrow_value, allowed_borrow_value);
        pack_decimal(self.unhealthy_borrow_value, unhealthy_borrow_value);
        *deposits_len = u8::try_from(self.deposits.len()).unwrap().to_le_bytes();
        *borrows_len = u8::try_from(self.borrows.len()).unwrap().to_le_bytes();
        pack_decimal(self.unclaimed_mine, unclaimed_mine);
        let mut offset = 0;

        // deposits
        for collateral in &self.deposits {
            let deposits_flat = array_mut_ref![data_flat, offset, OBLIGATION_COLLATERAL_LEN];
            #[allow(clippy::ptr_offset_with_cast)]
            let (
                deposit_reserve,
                deposited_amount,
                market_value,
                index
            ) = mut_array_refs![deposits_flat, PUBKEY_BYTES, 8, 16,16];
            deposit_reserve.copy_from_slice(collateral.deposit_reserve.as_ref());
            *deposited_amount = collateral.deposited_amount.to_le_bytes();
            pack_decimal(collateral.market_value, market_value);
            pack_decimal(collateral.index,index);
            offset += OBLIGATION_COLLATERAL_LEN;
        }

        // borrows
        for liquidity in &self.borrows {
            let borrows_flat = array_mut_ref![data_flat, offset, OBLIGATION_LIQUIDITY_LEN];
            #[allow(clippy::ptr_offset_with_cast)]
            let (borrow_reserve,
                cumulative_borrow_rate_wads,
                borrowed_amount_wads,
                market_value,
                index
            ) = mut_array_refs![borrows_flat, PUBKEY_BYTES, 16, 16, 16,16];
            borrow_reserve.copy_from_slice(liquidity.borrow_reserve.as_ref());
            pack_decimal(
                liquidity.cumulative_borrow_rate_wads,
                cumulative_borrow_rate_wads,
            );
            pack_decimal(liquidity.borrowed_amount_wads, borrowed_amount_wads);
            pack_decimal(liquidity.market_value, market_value);
            pack_decimal(liquidity.index,index);
            offset += OBLIGATION_LIQUIDITY_LEN;
        }
    }

    /// Unpacks a byte buffer into an [ObligationInfo](struct.ObligationInfo.html).
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let input = array_ref![src, 0, OBLIGATION_LEN];
        #[allow(clippy::ptr_offset_with_cast)]
        let (
            version,
            last_update_slot,
            last_update_stale,
            lending_market,
            owner,
            deposited_value,
            borrowed_value,
            allowed_borrow_value,
            unhealthy_borrow_value,
            deposits_len,
            borrows_len,
            unclaimed_mine,
            data_flat,
        ) = array_refs![
            input,
            1,
            8,
            1,
            PUBKEY_BYTES,
            PUBKEY_BYTES,
            16,
            16,
            16,
            16,
            1,
            1,
            16,
            OBLIGATION_COLLATERAL_LEN + (OBLIGATION_LIQUIDITY_LEN * (MAX_OBLIGATION_RESERVES - 1))
        ];

        let version = u8::from_le_bytes(*version);
        if version > PROGRAM_VERSION {
            msg!("Obligation version does not match lending program version");
            return Err(ProgramError::InvalidAccountData);
        }

        let deposits_len = u8::from_le_bytes(*deposits_len);
        let borrows_len = u8::from_le_bytes(*borrows_len);
        let mut deposits = Vec::with_capacity(deposits_len as usize + 1);
        let mut borrows = Vec::with_capacity(borrows_len as usize + 1);

        let mut offset = 0;
        for _ in 0..deposits_len {
            let deposits_flat = array_ref![data_flat, offset, OBLIGATION_COLLATERAL_LEN];
            #[allow(clippy::ptr_offset_with_cast)]
            let (
                deposit_reserve,
                deposited_amount,
                market_value,
                index
            ) = array_refs![deposits_flat, PUBKEY_BYTES, 8, 16, 16];
            deposits.push(ObligationCollateral {
                index: unpack_decimal(index),
                deposit_reserve: Pubkey::new(deposit_reserve),
                deposited_amount: u64::from_le_bytes(*deposited_amount),
                market_value: unpack_decimal(market_value),
            });

            offset += OBLIGATION_COLLATERAL_LEN;
        }
        for _ in 0..borrows_len {
            let borrows_flat = array_ref![data_flat, offset, OBLIGATION_LIQUIDITY_LEN];
            #[allow(clippy::ptr_offset_with_cast)]
            let (borrow_reserve,
                cumulative_borrow_rate_wads,
                borrowed_amount_wads,
                market_value,
                index
            ) = array_refs![borrows_flat, PUBKEY_BYTES, 16, 16, 16,16];
            borrows.push(ObligationLiquidity {
                index:unpack_decimal(index),
                borrow_reserve: Pubkey::new(borrow_reserve),
                cumulative_borrow_rate_wads: unpack_decimal(cumulative_borrow_rate_wads),
                borrowed_amount_wads: unpack_decimal(borrowed_amount_wads),
                market_value: unpack_decimal(market_value),
            });
            offset += OBLIGATION_LIQUIDITY_LEN;
        }

        Ok(Self {
            version,
            last_update: LastUpdate {
                slot: u64::from_le_bytes(*last_update_slot),
                stale: unpack_bool(last_update_stale)?,
            },
            lending_market: Pubkey::new_from_array(*lending_market),
            owner: Pubkey::new_from_array(*owner),
            deposits,
            borrows,
            deposited_value: unpack_decimal(deposited_value),
            borrowed_value: unpack_decimal(borrowed_value),
            allowed_borrow_value: unpack_decimal(allowed_borrow_value),
            unhealthy_borrow_value: unpack_decimal(unhealthy_borrow_value),
            unclaimed_mine: unpack_decimal(unclaimed_mine),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::math::TryAdd;
    use proptest::prelude::*;

    const MAX_COMPOUNDED_INTEREST: u64 = 100; // 10,000%

    #[test]
    fn obligation_accrue_interest_failure() {
        assert_eq!(
            ObligationLiquidity {
                cumulative_borrow_rate_wads: Decimal::zero(),
                ..ObligationLiquidity::default()
            }
                .accrue_interest(Decimal::one()),
            Err(LendingError::MathOverflow.into())
        );

        assert_eq!(
            ObligationLiquidity {
                cumulative_borrow_rate_wads: Decimal::from(2u64),
                ..ObligationLiquidity::default()
            }
                .accrue_interest(Decimal::one()),
            Err(LendingError::NegativeInterestRate.into())
        );

        assert_eq!(
            ObligationLiquidity {
                cumulative_borrow_rate_wads: Decimal::one(),
                borrowed_amount_wads: Decimal::from(u64::MAX),
                ..ObligationLiquidity::default()
            }
                .accrue_interest(Decimal::from(10 * MAX_COMPOUNDED_INTEREST)),
            Err(LendingError::MathOverflow.into())
        );
    }

    // Creates rates (r1, r2) where 0 < r1 <= r2 <= 100*r1
    prop_compose! {
        fn cumulative_rates()(rate in 1..=u128::MAX)(
            current_rate in Just(rate),
            max_new_rate in rate..=rate.saturating_mul(MAX_COMPOUNDED_INTEREST as u128),
        ) -> (u128, u128) {
            (current_rate, max_new_rate)
        }
    }

    const MAX_BORROWED: u128 = u64::MAX as u128 * WAD as u128;

    // Creates liquidity amounts (repay, borrow) where repay < borrow
    prop_compose! {
        fn repay_partial_amounts()(amount in 1..=u64::MAX)(
            repay_amount in Just(WAD as u128 * amount as u128),
            borrowed_amount in (WAD as u128 * amount as u128 + 1)..=MAX_BORROWED,
        ) -> (u128, u128) {
            (repay_amount, borrowed_amount)
        }
    }

    // Creates liquidity amounts (repay, borrow) where repay >= borrow
    prop_compose! {
        fn repay_full_amounts()(amount in 1..=u64::MAX)(
            repay_amount in Just(WAD as u128 * amount as u128),
        ) -> (u128, u128) {
            (repay_amount, repay_amount)
        }
    }

    proptest! {
        #[test]
        fn repay_partial(
            (repay_amount, borrowed_amount) in repay_partial_amounts(),
        ) {
            let borrowed_amount_wads = Decimal::from_scaled_val(borrowed_amount);
            let repay_amount_wads = Decimal::from_scaled_val(repay_amount);
            println!("borrowed_amount_wads=,repay_amount_wads");
            println!("{}",borrowed_amount_wads);
            println!("{}",repay_amount_wads);
            let mut obligation = Obligation {
                borrows: vec![ObligationLiquidity {
                    borrowed_amount_wads,
                    ..ObligationLiquidity::default()
                }],
                ..Obligation::default()
            };

            obligation.repay(repay_amount_wads, 0)?;
            assert!(obligation.borrows[0].borrowed_amount_wads < borrowed_amount_wads);
            assert!(obligation.borrows[0].borrowed_amount_wads > Decimal::zero());
        }

        #[test]
        fn repay_full(
            (repay_amount, borrowed_amount) in repay_full_amounts(),
        ) {
            let borrowed_amount_wads = Decimal::from_scaled_val(borrowed_amount);
            let repay_amount_wads = Decimal::from_scaled_val(repay_amount);
            let mut obligation = Obligation {
                borrows: vec![ObligationLiquidity {
                    borrowed_amount_wads,
                    ..ObligationLiquidity::default()
                }],
                ..Obligation::default()
            };

            obligation.repay(repay_amount_wads, 0)?;
            assert_eq!(obligation.borrows.len(), 0);
        }

        #[test]
        fn accrue_interest(
            (current_borrow_rate, new_borrow_rate) in cumulative_rates(),
            borrowed_amount in 0..=u64::MAX,
        ) {
            let cumulative_borrow_rate_wads = Decimal::one().try_add(Decimal::from_scaled_val(current_borrow_rate))?;
            let borrowed_amount_wads = Decimal::from(borrowed_amount);
            let mut liquidity = ObligationLiquidity {
                cumulative_borrow_rate_wads,
                borrowed_amount_wads,
                ..ObligationLiquidity::default()
            };

            let next_cumulative_borrow_rate = Decimal::one().try_add(Decimal::from_scaled_val(new_borrow_rate))?;
            liquidity.accrue_interest(next_cumulative_borrow_rate)?;

            if next_cumulative_borrow_rate > cumulative_borrow_rate_wads {
                assert!(liquidity.borrowed_amount_wads > borrowed_amount_wads);
            } else {
                assert!(liquidity.borrowed_amount_wads == borrowed_amount_wads);
            }
        }
    }
}
