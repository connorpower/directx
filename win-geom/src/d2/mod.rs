//! 2-Dimensional types optimized for Win32 and Direct2D APIs with some rusty
//! conveniences.
//!
//! # Conversions
//!
//! If _feature_ `"d2d"` is enabled, then some primitives can be directly
//! converted into a Direct2D structures.
//!
//! If _feature_ `"win32"` is enabled, then some primitives can be directly
//! converted into a Win32 structures.

use ::num_traits::{AsPrimitive, Num};
use ::std::{fmt::Debug, ops::Add};

#[cfg(feature = "d2d")]
pub use d2d::*;
#[cfg(feature = "win32")]
pub use win32::*;

/// 2D point representation, compatible with any numeric representation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct Point2D<T>
where
    T: Num + Clone + Copy + Debug,
{
    /// Co-ordinate along the x axis (horizontal).
    pub x: T,
    /// Co-ordinate along the y axis (vertical).
    pub y: T,
}

impl<T> Default for Point2D<T>
where
    T: Num + Clone + Copy + Debug,
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
    T: Num + Clone + Copy + Debug,
{
    /// Creates a new [`Point2D`] with `{x: 0, y: 0}` in whichever numeric type
    /// is specified by `T`.
    ///
    /// # Example
    ///
    /// ```
    /// use ::win_geom::d2::Point2D;
    ///
    /// let origin = Point2D::<f32>::zero();
    ///
    /// assert_eq!(origin.x, 0.0);
    /// assert_eq!(origin.y, 0.0);
    /// ```
    pub fn zero() -> Self {
        Self::default()
    }
}

/// 2D size representation, compatible with any numeric representation.
///
/// # Conversions
///
/// If _feature_ `"d2d"` is enabled, then a [`Size2D<u32>`] can be directly
/// converted into a Direct2D `D2D_SIZE_U` struct.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct Size2D<T>
where
    T: Num + Clone + Copy + Debug,
{
    /// The extent of the element along the x axis.
    pub width: T,
    /// The extent of the element along the y axis.
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
    /// Creates a new [`Size2D`] with `{width: 0, height: 0}` in whichever
    /// numeric type is specified by `T`.
    ///
    /// # Example
    ///
    /// ```
    /// use ::win_geom::d2::Size2D;
    ///
    /// let zero = Size2D::<f32>::zero();
    ///
    /// assert_eq!(zero.width, 0.0);
    /// assert_eq!(zero.height, 0.0);
    /// ```
    pub fn zero() -> Self {
        Self::default()
    }

    /// Creates a new [`Size2D`] with `{x: 1, y: 1}` in whichever numeric type
    /// is specified by `T`.
    ///
    /// # Example
    ///
    /// ```
    /// use ::win_geom::d2::Size2D;
    ///
    /// let pixel = Size2D::<f32>::pixel();
    ///
    /// assert_eq!(pixel.width, 1.0);
    /// assert_eq!(pixel.height, 1.0);
    /// ```
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
    /// A generic interface which casts a [`Size2D`] from numeric representation
    /// into another. The cast will never fail but may cause narrowing or
    /// precision loss. The underlying cast operates the same as the `as`
    /// keyword.
    ///
    /// # Example
    ///
    /// ```
    /// use ::win_geom::d2::{Rect2D, Size2D, Point2D};
    ///
    /// let size = Size2D { width: 10.3_f32, height: 10.8 };
    ///
    /// // Convert our float size into an integer size compatible with the
    /// // Win32 `RECT` class.
    /// let uint_size = size.cast::<u32>();
    ///
    /// assert_eq!(uint_size.width, 10);
    /// ```
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

/// 2D dimensional rectangle, compatible with any numeric representation.
///
/// # Conversions
///
/// If _feature_ `"d2d"` is enabled, then a [`Rect2D<f32>`] can be directly
/// converted into a Direct2D `D2D_RECT_F` struct.
///
/// If _feature_ `"win32"` is enabled, then a [`Rect2D<u32>`] can be directly
/// converted into a Win32 `RECT` struct.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct Rect2D<T>
where
    T: Num + Clone + Copy + Debug,
{
    /// The left-most edge, or minimum x value.
    pub left: T,
    /// The top-most edge, or minimum y value.
    pub top: T,
    /// The right-most edge, or maximum x value.
    pub right: T,
    /// The bottom-most edge, or maximum y value.
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
    /// Creates a new [`Rect2D`] with zero area in whichever numeric
    /// type is specified by `T`.
    ///
    /// # Example
    ///
    /// ```
    /// use ::win_geom::d2::Rect2D;
    ///
    /// let empty = Rect2D::<f32>::zero();
    ///
    /// assert_eq!(empty.left, 0.0);
    /// assert_eq!(empty.right, 0.0);
    /// assert_eq!(empty.top, 0.0);
    /// assert_eq!(empty.bottom, 0.0);
    /// ```
    pub fn zero() -> Self {
        Self::default()
    }
}

impl<T> Rect2D<T>
where
    T: Num + Clone + Copy + Debug,
{
    /// Constructs a [`Rect2D`] with a given [`Size2D`], anchored with the
    /// origin (top-left corner) rooted at `origin`.
    ///
    /// # Example
    ///
    /// ```
    /// use ::win_geom::d2::{Rect2D, Size2D, Point2D};
    ///
    /// let rect = Rect2D::<f32>::from_size_and_origin(
    ///     Size2D {
    ///         width: 10.0,
    ///         height: 10.0
    ///     },
    ///     Point2D {
    ///         x: 2.5,
    ///         y: 5.0,
    ///     },
    /// );
    ///
    /// assert_eq!(rect.left, 2.5);
    /// assert_eq!(rect.right, 12.5);
    /// assert_eq!(rect.top, 5.0);
    /// assert_eq!(rect.bottom, 15.0);
    /// ```
    pub fn from_size_and_origin(size: Size2D<T>, origin: Point2D<T>) -> Self
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

    /// Returns the width of the rect.
    pub fn width(&self) -> T {
        self.right - self.left
    }

    /// Returns the height of the rect.
    pub fn height(&self) -> T {
        self.bottom - self.top
    }

    /// A generic interface which casts a [`Rect2D`] from numeric representation
    /// into another. The cast will never fail but may cause narrowing or
    /// precision loss. The underlying cast operates the same as the `as`
    /// keyword.
    ///
    /// # Example
    ///
    /// ```
    /// use ::win_geom::d2::{Rect2D, Size2D, Point2D};
    ///
    /// let float_rect = Rect2D::<f32>::from_size_and_origin(
    ///     Size2D {
    ///         width: 10.0,
    ///         height: 10.0
    ///     },
    ///     Point2D::zero(),
    /// );
    ///
    /// // Convert our float rect into an integer rect compatible with the
    /// // Win32 `RECT` class.
    /// let int_rect = float_rect.cast::<i32>();
    ///
    /// assert_eq!(int_rect.right, 10_i32);
    /// ```
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

/// 2D dimensional rounded rectangle, compatible with any numeric
/// representation. Contains the dimensions and corner radii of a rounded
/// rectangle.
///
/// Each corner of the rectangle specified by rect is replaced with a quarter
/// ellipse, with a radius in each direction specified by radiusX and radiusY.
///
/// If [`radius_x`] is greater than or equal to half the width of the rectangle,
/// and [`radius_y`] is greater than or equal to one-half the height, then the
/// rounded rectangle is an ellipse with the same width and height of rect.
///
/// # Direct2D Note
///
/// Even when both [`radius_x`] and [`radius_y`] are zero, a [`RoundedRect2D`]
/// is different from a [`Rect2D`]. When stroked, the corners of the rounded
/// rectangle are roundly joined, not mitered (square).
///
/// # Conversions
///
/// If _feature_ `"d2d"` is enabled, then a [`RoundedRect2D<f32>`] can be
/// directly converted into a Direct2D `D2D1_ROUNDED_RECT ` struct.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct RoundedRect2D<T>
where
    T: Num + Clone + Copy + Debug,
{
    /// The coordinates of the base rectangle.
    pub rect: Rect2D<T>,
    /// The x-radius for the quarter ellipse that is drawn to replace every
    /// corner of the rectangle.
    pub radius_x: T,
    /// The y-radius for the quarter ellipse that is drawn to replace every
    /// corner of the rectangle.
    pub radius_y: T,
}

impl<T> Default for RoundedRect2D<T>
where
    T: Num + Clone + Copy + Debug,
{
    fn default() -> Self {
        Self {
            rect: Rect2D::zero(),
            radius_x: T::zero(),
            radius_y: T::zero(),
        }
    }
}

impl<T> RoundedRect2D<T>
where
    T: Num + Clone + Copy + Debug,
{
    /// Creates a new [`RoundedRect2D`] with zero area in whichever numeric
    /// type is specified by `T`.
    ///
    /// # Example
    ///
    /// ```
    /// use ::win_geom::d2::RoundedRect2D;
    ///
    /// let empty = RoundedRect2D::<f32>::zero();
    ///
    /// assert_eq!(empty.rect.left, 0.0);
    /// assert_eq!(empty.rect.right, 0.0);
    /// assert_eq!(empty.rect.top, 0.0);
    /// assert_eq!(empty.rect.bottom, 0.0);
    /// assert_eq!(empty.radius_x, 0.0);
    /// assert_eq!(empty.radius)y, 0.0);
    /// ```
    pub fn zero() -> Self {
        Self::default()
    }
}

impl<T> RoundedRect2D<T>
where
    T: Num + Clone + Copy + Debug,
{
    /// Constructs a [`Rect2D`] with a given [`Size2D`], anchored with the
    /// origin (top-left corner) rooted at `origin`.
    ///
    /// # Example
    ///
    /// ```
    /// use ::win_geom::d2::{Rect2D, Size2D, Point2D};
    ///
    /// let rect = Rect2D::<f32>::from_size_and_origin(
    ///     Size2D {
    ///         width: 10.0,
    ///         height: 10.0
    ///     },
    ///     Point2D {
    ///         x: 2.5,
    ///         y: 5.0,
    ///     },
    /// );
    ///
    /// assert_eq!(rect.left, 2.5);
    /// assert_eq!(rect.right, 12.5);
    /// assert_eq!(rect.top, 5.0);
    /// assert_eq!(rect.bottom, 15.0);
    /// ```
    pub fn from_size_and_origin(size: Size2D<T>, origin: Point2D<T>, corner_radius: T) -> Self
    where
        T: Add<Output = T>,
    {
        Self {
            rect: Rect2D::from_size_and_origin(size, origin),
            radius_x: corner_radius,
            radius_y: corner_radius,
        }
    }

    /// Returns the width of the rect.
    pub fn width(&self) -> T {
        self.rect.width()
    }

    /// Returns the height of the rect.
    pub fn height(&self) -> T {
        self.rect.height()
    }

    /// A generic interface which casts a [`RoundedRect2D`] from numeric
    /// representation into another. The cast will never fail but may cause
    /// narrowing or precision loss. The underlying cast operates the same as
    /// the `as` keyword.
    ///
    /// # Example
    ///
    /// ```
    /// use ::win_geom::d2::{RoundedRect2D, Size2D, Point2D};
    ///
    /// let float_rect = RoundedRect2D::<f32>::from_size_and_origin(
    ///     Size2D {
    ///         width: 10.0,
    ///         height: 10.0
    ///     },
    ///     Point2D::zero(),
    ///     8.5,
    /// );
    ///
    /// // Convert our float rounded rect into an integer rounded rect.
    /// let int_rect = float_rect.cast::<i32>();
    ///
    /// assert_eq!(int_rect.radius_x, 8_i32);
    /// ```
    pub fn cast<U>(self) -> RoundedRect2D<U>
    where
        T: AsPrimitive<U>,
        U: Num + Clone + Copy + Debug + 'static,
    {
        RoundedRect2D::<U> {
            rect: self.rect.cast(),
            radius_x: self.radius_x.as_(),
            radius_y: self.radius_y.as_(),
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
    use ::windows::Win32::Graphics::Direct2D::{
        Common::{D2D_POINT_2F, D2D_RECT_F, D2D_SIZE_U},
        D2D1_ROUNDED_RECT,
    };

    impl From<Point2D<f32>> for D2D_POINT_2F {
        fn from(val: Point2D<f32>) -> Self {
            // SAFETY: our `Point2D` is modelled on the same memory layout as
            // the Direct2D `D2D_POINT_2F` and we restrict this conversion
            // implementation to sizes with `f32` representations.
            unsafe { ::std::mem::transmute(val) }
        }
    }

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

    impl From<RoundedRect2D<f32>> for D2D1_ROUNDED_RECT {
        fn from(val: RoundedRect2D<f32>) -> Self {
            // SAFETY: our `RoundedRect2D` is modelled on the same memory layout
            // as the Direct2D `D2D1_ROUNDED_RECT` and we restrict this
            // conversion implementation to rectangles with `f32`
            // representations.
            unsafe { ::std::mem::transmute(val) }
        }
    }
}
