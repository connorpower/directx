//! [`::d2d`](crate) is a Direct2D-based graphics package which provides only
//! the most minimal of conveniences over the underlying DirectX implementation.

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

mod color;
mod context;
mod factory;
mod target;

pub use color::*;
pub use context::*;
pub use factory::*;
pub use target::*;
