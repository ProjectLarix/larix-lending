// #![deny(missing_docs)]

//! A lending program for the Solana blockchain.

// Export current sdk types for downstream users building with a different sdk version
pub use solana_program;

pub mod error;
pub mod instruction;
pub mod math;
pub mod state;
pub mod config;
pub mod util;


solana_program::declare_id!("7Zb1bGi32pfsrBkzWdqd4dFhUXwp5Nybr1zuaEwN34hy");
