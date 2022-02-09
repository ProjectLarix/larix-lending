use super::*;
use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use solana_program::{
    msg,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::{Pubkey, PUBKEY_BYTES},
};

/// Lending market state
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LendingMarket {
    /// Version of lending market
    pub version: u8,
    /// Bump seed for derived authority address
    pub bump_seed: u8,
    /// The pending owner
    pub pending_owner:Pubkey,
    /// Owner authority which can add new reserves
    pub owner: Pubkey,
    /// Currency market prices are quoted in
    /// e.g. "USD" null padded (`*b"USD\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0"`) or a SPL token mint pubkey
    pub quote_currency: [u8; 32],
    /// Token program id
    pub token_program_id: Pubkey,
    /// Oracle (Pyth) program id
    pub oracle_program_id: Pubkey,
    /// Oracle (Larix) program id
    pub larix_oracle_program_id: Pubkey,
    /// Oracle id
    pub larix_oracle_id: Pubkey,
    /// Mint address of the mine token
    pub mine_mint: Pubkey,
    /// Supply address of mine token
    pub mine_supply_account: Pubkey,
    /// Larix lock program
    pub mine_lock_program: Pubkey,
    /// subsidy times to lock time. lock time = subsidy_times * lock_larix_times_to_time
    pub lock_larix_times_to_time: u64,
    /// 200 means max claim 2 times and must equals or great than 100
    pub max_claim_times: u16,

}

impl LendingMarket {
    /// Create a new lending market
    pub fn new(params: InitLendingMarketParams) -> Self {
        let mut lending_market = Self::default();
        Self::init(&mut lending_market, params);
        lending_market
    }

    /// Initialize a lending market
    pub fn init(&mut self, params: InitLendingMarketParams) {
        self.version = PROGRAM_VERSION;
        self.bump_seed = params.bump_seed;
        self.pending_owner = Pubkey::default();
        self.owner = params.owner;
        self.quote_currency = params.quote_currency;
        self.token_program_id = params.token_program_id;
        self.oracle_program_id = params.oracle_program_id;
        self.larix_oracle_program_id = params.larix_oracle_program_id;
        self.larix_oracle_id = params.larix_oracle_id;
        self.mine_mint = params.mine_mint;
        self.mine_supply_account = params.mine_supply_account;
        self.mine_lock_program = params.mine_lock_program;
    }
}

/// Initialize a lending market
pub struct InitLendingMarketParams {
    /// Bump seed for derived authority address
    pub bump_seed: u8,
    /// Owner authority which can add new reserves
    pub owner: Pubkey,
    /// Currency market prices are quoted in
    /// e.g. "USD" null padded (`*b"USD\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0"`) or a SPL token mint pubkey
    pub quote_currency: [u8; 32],
    /// Token program id
    pub token_program_id: Pubkey,
    /// Oracle (Pyth) program id
    pub oracle_program_id: Pubkey,
    /// Oracle (Larix) program id
    pub larix_oracle_program_id: Pubkey,
    /// Oracle id
    pub larix_oracle_id: Pubkey,
    /// Mint address of the mine token
    pub mine_mint: Pubkey,
    /// Supply address of mine token
    pub mine_supply_account: Pubkey,
    /// Larix lock program
    pub mine_lock_program: Pubkey,

}

impl Sealed for LendingMarket {}
impl IsInitialized for LendingMarket {
    fn is_initialized(&self) -> bool {
        self.version != UNINITIALIZED_VERSION
    }
}

const LENDING_MARKET_LEN: usize = 418; // 1 + 1 + 32 + 32 + 32 + 32 + 32 + 128
impl Pack for LendingMarket {
    const LEN: usize = LENDING_MARKET_LEN;

    fn pack_into_slice(&self, output: &mut [u8]) {
        let output = array_mut_ref![output, 0, LENDING_MARKET_LEN];
        #[allow(clippy::ptr_offset_with_cast)]
        let (
            version,
            bump_seed,
            pending_owner,
            owner,
            quote_currency,
            token_program_id,
            oracle_program_id,
            larix_oracle_program_id,
            larix_oracle_id,
            mine_mint,
            mine_supply_account,
            mine_lock_program,
            lock_larix_times_to_time,
            max_claim_times,
            _padding
        ) = mut_array_refs![
            output,
            1,
            1,
            PUBKEY_BYTES,
            PUBKEY_BYTES,
            PUBKEY_BYTES,
            PUBKEY_BYTES,
            PUBKEY_BYTES,
            PUBKEY_BYTES,
            PUBKEY_BYTES,
            PUBKEY_BYTES,
            PUBKEY_BYTES,
            PUBKEY_BYTES,
            8,
            2,
            86
        ];

        *version = self.version.to_le_bytes();
        *bump_seed = self.bump_seed.to_le_bytes();
        owner.copy_from_slice(self.owner.as_ref());
        quote_currency.copy_from_slice(self.quote_currency.as_ref());
        token_program_id.copy_from_slice(self.token_program_id.as_ref());
        oracle_program_id.copy_from_slice(self.oracle_program_id.as_ref());
        larix_oracle_program_id.copy_from_slice(self.larix_oracle_program_id.as_ref());
        pending_owner.copy_from_slice(self.pending_owner.as_ref());
        larix_oracle_id.copy_from_slice(self.larix_oracle_id.as_ref());
        mine_mint.copy_from_slice(self.mine_mint.as_ref());
        mine_supply_account.copy_from_slice(self.mine_supply_account.as_ref());
        mine_lock_program.copy_from_slice(self.mine_lock_program.as_ref());
        *lock_larix_times_to_time = self.lock_larix_times_to_time.to_le_bytes();
        *max_claim_times = self.max_claim_times.to_le_bytes();

    }

    /// Unpacks a byte buffer into a [LendingMarketInfo](struct.LendingMarketInfo.html)
    fn unpack_from_slice(input: &[u8]) -> Result<Self, ProgramError> {
        let input = array_ref![input, 0, LENDING_MARKET_LEN];
        #[allow(clippy::ptr_offset_with_cast)]
        let (
            version,
            bump_seed,
            pending_owner,
            owner,
            quote_currency,
            token_program_id,
            oracle_program_id,
            larix_oracle_program_id,
            larix_oracle_id,
            mine_mint,
            mine_supply_account,
            mine_lock_program,
            lock_larix_times_to_time,
            max_claim_times,
            _padding,
        ) = array_refs![
            input,
            1,
            1,
            PUBKEY_BYTES,
            PUBKEY_BYTES,
            PUBKEY_BYTES,
            PUBKEY_BYTES,
            PUBKEY_BYTES,
            PUBKEY_BYTES,
            PUBKEY_BYTES,
            PUBKEY_BYTES,
            PUBKEY_BYTES,
            PUBKEY_BYTES,
            8,
            2,
            86
        ];

        let version = u8::from_le_bytes(*version);
        if version > PROGRAM_VERSION {
            msg!("Lending market version does not match lending program version");
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(Self {
            version,
            bump_seed: u8::from_le_bytes(*bump_seed),
            pending_owner: Pubkey::new_from_array(*pending_owner),
            owner: Pubkey::new_from_array(*owner),
            quote_currency: *quote_currency,
            token_program_id: Pubkey::new_from_array(*token_program_id),
            oracle_program_id: Pubkey::new_from_array(*oracle_program_id),
            larix_oracle_program_id: Pubkey::new_from_array(*larix_oracle_program_id),
            larix_oracle_id: Pubkey::new_from_array(*larix_oracle_id),
            mine_mint: Pubkey::new_from_array(*mine_mint),
            mine_supply_account: Pubkey::new_from_array(*mine_supply_account),
            mine_lock_program: Pubkey::new_from_array(*mine_lock_program),
            lock_larix_times_to_time: u64::from_le_bytes(*lock_larix_times_to_time),
            max_claim_times: u16::from_le_bytes(*max_claim_times),
        })
    }
}
