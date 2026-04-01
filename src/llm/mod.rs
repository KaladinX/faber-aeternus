pub mod provider;
pub use provider::{LLMProvider, ProviderConfig};

#[cfg(feature = "reqwest")]
pub mod remote;

#[cfg(feature = "candle-core")]
pub mod local;
