//! # Prompt Compiler Core
//!
//! Core compiler logic and intermediate representation (IR) for the prompt compiler.
//! This crate provides the fundamental compilation pipeline and data structures.

pub mod compiler;
pub mod error;
pub mod ir;

pub use compiler::*;
pub use error::*;
pub use ir::*;

/// Version information for the core library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
