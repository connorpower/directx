//! Geometry primitives with memory layouts optimized for native Windows APIs
//! (Win32, Direct2D, and Direct3D).
//!
//! # Conversions
//!
//! If _feature_ `"d2d"` is enabled, then some primitives can be directly
//! converted into a Direct2D structures.
//!
//! If _feature_ `"win32"` is enabled, then some primitives can be directly
//! converted into a Win32 structures.

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

pub mod d2;
