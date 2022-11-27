use ::num_traits::{Num, NumCast, ToPrimitive};
use ::std::fmt::Debug;

#[cfg(feature = "d2d")]
pub use d2d::*;
#[cfg(feature = "win32")]
pub use win32::*;

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

impl<T> Dimension2D<T>
where
    T: ToPrimitive + Num + Clone + Copy + Debug,
{
    pub fn map<U>(self) -> Option<Dimension2D<U>>
    where
        U: NumCast + Num + Clone + Copy + Debug,
    {
        Some(Dimension2D::<U> {
            width: U::from(self.width)?,
            height: U::from(self.height)?,
        })
    }
}

#[cfg(feature = "win32")]
mod win32 {
    use super::*;
    use ::windows::Win32::Foundation::RECT;

    impl From<Dimension2D<i32>> for RECT {
        fn from(s: Dimension2D<i32>) -> Self {
            Self {
                left: 0,
                top: 0,
                right: s.width,
                bottom: s.height,
            }
        }
    }
}

#[cfg(feature = "d2d")]
mod d2d {
    use super::*;
    use ::windows::Win32::Graphics::Direct2D::Common::D2D_SIZE_U;

    impl<T> From<Dimension2D<T>> for D2D_SIZE_U
    where
        T: Num + Clone + Copy + Debug,
        u32: From<T>,
    {
        fn from(s: Dimension2D<T>) -> Self {
            Self {
                width: s.width.into(),
                height: s.height.into(),
            }
        }
    }
}
