//! Arithmetic utilities.

use core::ops::{Add, Div, Rem};

use num_traits::{CheckedAdd, CheckedDiv, CheckedRem, One, Zero};

/// Provides ability to perform ceiling division operations on integers.
pub trait DivCeil: Sized {
	/// Performs ceiling division usign supplied operands.
	fn div_ceil(self, other: Self) -> Self;
}

/// Provides ability to perform checked ceiling division operations on integers.
pub trait CheckedDivCeil: Sized {
	/// Performs checked ceiling division usign supplied operands.
	///
	/// Returns `None` in case either divider is zero or the calculation overflowed.
	fn checked_div_ceil(self, other: Self) -> Option<Self>;
}

/// Implements `DivCeil` for any type which implements `Div`/`Rem`/`Add`/`Ord`/`Zero`/`One`/`Copy`.
impl<T> DivCeil for T
where
	T: Div<Output = T> + Rem<Output = T> + Add<Output = T> + Ord + Zero + One + Copy,
{
	fn div_ceil(self, other: Self) -> Self {
		let quot = self / other;
		let rem = self % other;
		let zero = Self::zero();

		if (rem > zero && other > zero) || (rem < zero && other < zero) {
			quot + One::one()
		} else {
			quot
		}
	}
}

/// Implements `CheckedDivCeil` for any type which implements
/// `CheckedDiv`/`CheckedRem`/`CheckedAdd`/`Ord`/`Zero`/`One`/`Copy`.
impl<T> CheckedDivCeil for T
where
	T: CheckedDiv + CheckedRem + CheckedAdd + Ord + Zero + One + Copy,
{
	fn checked_div_ceil(self, other: Self) -> Option<Self> {
		let quot = self.checked_div(&other)?;
		let rem = self.checked_rem(&other)?;
		let zero = Self::zero();

		if (rem > zero && other > zero) || (rem < zero && other < zero) {
			quot.checked_add(&One::one())
		} else {
			Some(quot)
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn div_ceil() {
		assert_eq!(9.div_ceil(2), 5);
		assert_eq!(10.div_ceil(2), 5);
		assert_eq!(11.div_ceil(2), 6);
		assert_eq!(12.div_ceil(2), 6);
		assert_eq!(0.div_ceil(1), 0);
		assert_eq!(1.div_ceil(1), 1);
	}

	#[test]
	fn checked_div_ceil() {
		assert_eq!(9.checked_div_ceil(2), Some(5));
		assert_eq!(10.checked_div_ceil(2), Some(5));
		assert_eq!(11.checked_div_ceil(2), Some(6));
		assert_eq!(12.checked_div_ceil(2), Some(6));
		assert_eq!(0.checked_div_ceil(1), Some(0));
		assert_eq!(1.checked_div_ceil(1), Some(1));
		assert_eq!(1.checked_div_ceil(0), None);
	}

	#[test]
	fn div_ceil_negative() {
		assert_eq!((0).div_ceil(-1), 0);
		assert_eq!((-1).div_ceil(2), 0);
		assert_eq!((-9).div_ceil(2), -4);
		assert_eq!((-10).div_ceil(2), -5);
		assert_eq!((-11).div_ceil(2), -5);
		assert_eq!((-12).div_ceil(2), -6);
		assert_eq!(0.div_ceil(1), 0);
		assert_eq!((-1).div_ceil(1), -1);

		assert_eq!((-1).div_ceil(-2), 1);
		assert_eq!((-9).div_ceil(-2), 5);
		assert_eq!((-10).div_ceil(-2), 5);
		assert_eq!((-11).div_ceil(-2), 6);
		assert_eq!((-12).div_ceil(-2), 6);
		assert_eq!(0.div_ceil(-1), 0);
		assert_eq!((-1).div_ceil(-1), 1);
	}

	#[test]
	fn checked_div_ceil_negative() {
		assert_eq!((0).checked_div_ceil(-1), Some(0));
		assert_eq!((-1).checked_div_ceil(2), Some(0));
		assert_eq!((-9).checked_div_ceil(2), Some(-4));
		assert_eq!((-10).checked_div_ceil(2), Some(-5));
		assert_eq!((-11).checked_div_ceil(2), Some(-5));
		assert_eq!((-12).checked_div_ceil(2), Some(-6));
		assert_eq!(0.checked_div_ceil(1), Some(0));
		assert_eq!(1.checked_div_ceil(0), None);
		assert_eq!((-1).checked_div_ceil(1), Some(-1));

		assert_eq!((-1).checked_div_ceil(-2), Some(1));
		assert_eq!((-9).checked_div_ceil(-2), Some(5));
		assert_eq!((-10).checked_div_ceil(-2), Some(5));
		assert_eq!((-11).checked_div_ceil(-2), Some(6));
		assert_eq!((-12).checked_div_ceil(-2), Some(6));
		assert_eq!(0.checked_div_ceil(-1), Some(0));
		assert_eq!((-1).checked_div_ceil(-0), None);
		assert_eq!((-1).checked_div_ceil(-1), Some(1));
	}
}
