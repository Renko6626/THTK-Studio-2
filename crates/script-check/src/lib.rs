//! Shared context and external truth process execution; no script parser or GUI dependency.
mod bundled_maps;
pub mod catalog;
mod context;
pub mod diagnostics;
mod runner;
mod types;
pub use context::*;
pub use runner::run;
pub use types::*;
