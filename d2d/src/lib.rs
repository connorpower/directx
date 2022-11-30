//! [`::d2d`] is a Direct2D-based graphics package which provides only the most
//! minimal of conveniences over the underlying DirectX implementation.

mod color;
mod context;
mod target;

pub use color::*;
pub use context::*;
pub use target::*;
