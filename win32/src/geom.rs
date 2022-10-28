//! Geometry primitives and functions for working with them.

use ::std::fmt::Debug;
use ::windows::Win32::Foundation::RECT;
use ::num_traits::Num;

/// 2D point representation, compatible with any numeric type.
#[derive(Clone, Copy, Debug)]
pub struct Point2D<T>
where
    T: Num + Clone + Copy + Debug,
{
    pub x: T,
    pub y: T,
}

/// 2D dimension representation, compatible with any numeric type.
#[derive(Clone, Copy, Debug)]
pub struct Dimension2D<T>
where
    T: Num + Clone + Copy + Debug,
{
    pub width: T,
    pub height: T,
}

impl From<Dimension2D<i32>> for RECT
{
    fn from(s: Dimension2D<i32>) -> Self {
        Self {
            left: 0,
            top: 0,
            right: s.width,
            bottom: s.height,
        }
    }
}
