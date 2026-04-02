// src/brain/specialists/mod.rs
pub mod architect;
pub mod coder;
pub mod reviewer;

pub use architect::ArchitectAgent;
pub use coder::CoderAgent;
pub use reviewer::ReviewerAgent;
