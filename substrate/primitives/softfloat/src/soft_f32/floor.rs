use super::F32;

/// Floor (F32)
///
/// Finds the nearest integer less than or equal to `x`.
pub const fn floor(x: F32) -> F32 {
	let mut ui = x.to_bits();
	let e = (((ui >> 23) as i32) & 0xff) - 0x7f;

	if e >= 23 {
		return x;
	}
	if e >= 0 {
		let m: u32 = 0x007fffff >> e;
		if (ui & m) == 0 {
			return x;
		}
		// force_eval!(x + F32::from_bits(0x7b800000));
		if ui >> 31 != 0 {
			ui += m;
		}
		ui &= !m;
	} else {
		// force_eval!(x + F32::from_bits(0x7b800000));
		if ui >> 31 == 0 {
			ui = 0;
		} else if ui << 1 != 0 {
			return f32!(-1.0);
		}
	}
	F32::from_bits(ui)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn sanity_check() {
		assert_eq!(floor(f32!(0.5)), f32!(0.0));
		assert_eq!(floor(f32!(1.1)), f32!(1.0));
		assert_eq!(floor(f32!(2.9)), f32!(2.0));
	}

	/// The spec: https://en.cppreference.com/w/cpp/numeric/math/floor
	#[test]
	fn spec_tests() {
		// Not Asserted: that the current rounding mode has no effect.
		assert!(floor(f32!(f32::NAN)).to_native_f32().is_nan());
		for f in [0.0, -0.0, f32::INFINITY, f32::NEG_INFINITY].iter().copied() {
			assert_eq!(F32::from_native_f32(f).floor(), F32::from_native_f32(f));
		}
	}
}
