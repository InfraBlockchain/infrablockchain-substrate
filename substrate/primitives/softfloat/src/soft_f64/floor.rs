use super::{
	helpers::{eq, gt},
	F64,
};

const TOINT: F64 = f64!(1.0).div(f64!(f64::EPSILON));

/// Floor (f64)
///
/// Finds the nearest integer less than or equal to `x`.
pub const fn floor(x: F64) -> F64 {
	let ui = x.to_bits();
	let e = ((ui >> 52) & 0x7ff) as i32;

	if (e >= 0x3ff + 52) || eq(&x, &F64::ZERO) {
		return x
	}
	/* y = int(x) - x, where int(x) is an integer neighbor of x */
	let y = if (ui >> 63) != 0 {
		x.sub(TOINT).add(TOINT).sub(x)
	} else {
		x.add(TOINT).sub(TOINT).sub(x)
	};
	/* special case because of non-nearest rounding modes */
	if e < 0x3ff {
		return if (ui >> 63) != 0 { f64!(-1.0) } else { F64::ZERO };
	}
	if gt(&y, &F64::ZERO) {
		x.add(y).sub(F64::ONE)
	} else {
		x.add(y)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn sanity_check() {
		assert_eq!(floor(f64!(1.1)), f64!(1.0));
		assert_eq!(floor(f64!(2.9)), f64!(2.0));
	}

	/// The spec: https://en.cppreference.com/w/cpp/numeric/math/floor
	#[test]
	fn spec_tests() {
		// Not Asserted: that the current rounding mode has no effect.
		assert!(floor(f64!(f64::NAN)).to_native_f64().is_nan());
		for f in [0.0, -0.0, f64::INFINITY, f64::NEG_INFINITY].iter().copied() {
			assert_eq!(floor(F64::from_native_f64(f)).to_native_f64(), f);
		}
	}
}
