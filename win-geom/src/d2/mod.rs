//! Geometry types that are optimized for compatibility with Win32 and Direct2D
//! representations.

use ::num_traits::{AsPrimitive, Num};
use ::std::{fmt::Debug, ops::Add};

#[cfg(feature = "d2d")]
pub use d2d::*;
#[cfg(feature = "win32")]
pub use win32::*;

/// 2D point representation.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Point2D<T>
where
    T: Num + Clone + Copy + Debug,
{
    pub x: T,
    pub y: T,
}

impl<T> Default for Point2D<T>
where
    T: Num + Clone + Copy + Debug + Default,
{
    fn default() -> Self {
        Self {
            x: T::zero(),
            y: T::zero(),
        }
    }
}

impl<T> Point2D<T>
where
    T: Num + Clone + Copy + Debug + Default,
{
    pub fn zero() -> Self {
        Self::default()
    }
}

/// 2D size representation.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Size2D<T>
where
    T: Num + Clone + Copy + Debug,
{
    pub width: T,
    pub height: T,
}

impl<T> Default for Size2D<T>
where
    T: Num + Clone + Copy + Debug,
{
    fn default() -> Self {
        Self {
            width: T::zero(),
            height: T::zero(),
        }
    }
}

impl<T> Size2D<T>
where
    T: Num + Clone + Copy + Debug,
{
    pub fn zero() -> Self {
        Self::default()
    }

    pub fn pixel() -> Self {
        Self {
            width: T::one(),
            height: T::one(),
        }
    }
}

impl<T> Size2D<T>
where
    T: Num + Clone + Copy + Debug,
{
    pub fn cast<U>(self) -> Size2D<U>
    where
        T: AsPrimitive<U>,
        U: Num + Clone + Copy + Debug + 'static,
    {
        Size2D::<U> {
            width: self.width.as_(),
            height: self.height.as_(),
        }
    }
}

/// 2D dimensional rectangle.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Rect2D<T>
where
    T: Num + Clone + Copy + Debug,
{
    pub left: T,
    pub top: T,
    pub right: T,
    pub bottom: T,
}

impl<T> Default for Rect2D<T>
where
    T: Num + Clone + Copy + Debug,
{
    fn default() -> Self {
        Self {
            left: T::zero(),
            top: T::zero(),
            right: T::zero(),
            bottom: T::zero(),
        }
    }
}

impl<T> Rect2D<T>
where
    T: Num + Clone + Copy + Debug,
{
    pub fn zero() -> Self {
        Self::default()
    }
}

impl<T> Rect2D<T>
where
    T: Num + Clone + Copy + Debug,
{
    /// Constructs a [Rect2D] with a given [Size2D], anchored with the origin
    /// (top-left corner) rooted at `origin`.
    pub fn from_size_with_origin(size: Size2D<T>, origin: Point2D<T>) -> Self
    where
        T: Add<Output = T>,
    {
        Self {
            left: origin.x,
            top: origin.y,
            right: origin.x + size.width,
            bottom: origin.y + size.height,
        }
    }

    pub fn cast<U>(self) -> Rect2D<U>
    where
        T: AsPrimitive<U>,
        U: Num + Clone + Copy + Debug + 'static,
    {
        Rect2D::<U> {
            left: self.left.as_(),
            top: self.top.as_(),
            right: self.right.as_(),
            bottom: self.bottom.as_(),
        }
    }
}

#[cfg(feature = "win32")]
mod win32 {
    use super::*;
    use ::windows::Win32::Foundation::RECT;

    impl From<Rect2D<i32>> for RECT {
        fn from(val: Rect2D<i32>) -> Self {
            // SAFETY: our `Rect2D` is modelled on the same memory layout as the
            // windows `RECT` and we restrict this conversion implementation to
            // rectangles with `i32` representations.
            unsafe { ::std::mem::transmute(val) }
        }
    }
}

#[cfg(feature = "d2d")]
mod d2d {
    use super::*;
    use ::windows::Win32::Graphics::Direct2D::Common::{D2D_RECT_F, D2D_SIZE_U};

    impl From<Size2D<u32>> for D2D_SIZE_U {
        fn from(val: Size2D<u32>) -> Self {
            // SAFETY: our `Size2D` is modelled on the same memory layout as the
            // Direct2D `D2D_SIZE_U` and we restrict this conversion
            // implementation to sizes with `u32` representations.
            unsafe { ::std::mem::transmute(val) }
        }
    }

    impl From<Rect2D<f32>> for D2D_RECT_F {
        fn from(val: Rect2D<f32>) -> Self {
            // SAFETY: our `Rect2D` is modelled on the same memory layout as the
            // Direct2D `D2D_RECT_F` and we restrict this conversion
            // implementation to rectangles with `f32` representations.
            unsafe { ::std::mem::transmute(val) }
        }
    }
}
