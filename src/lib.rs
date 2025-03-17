// Solana Rust DEX - Main Library File

pub mod entrypoint;
pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;

// Export the program's entrypoint
#[cfg(not(feature = "no-entrypoint"))]
pub use entrypoint::process_instruction;

// Program ID
solana_program::declare_id!("DEX1111111111111111111111111111111111111111");
