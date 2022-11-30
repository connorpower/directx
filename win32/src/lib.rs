//! Utilities for interacting with Win32 API.

#![deny(rust_2018_idioms)]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![cfg_attr(
    doc,
    warn(
        rustdoc::bare_urls,
        rustdoc::broken_intra_doc_links,
        rustdoc::invalid_codeblock_attributes,
        rustdoc::invalid_rust_codeblocks,
        rustdoc::missing_crate_level_docs,
    )
)]

pub mod debug;
pub mod errors;
pub mod input;
pub mod invoke;
pub mod types;
pub mod window;

pub use errors::*;
pub use types::*;
