use super::F32;

pub(crate) const fn round(x: F32) -> F32 {
	F32::trunc(x.add(F32::copysign(f32!(0.5).sub(f32!(0.25).mul(f32!(f32::EPSILON))), x)))
}

#[cfg(test)]
mod tests {
	use super::F32;

	#[test]
	fn negative_zero() {
		assert_eq!(F32::round(f32!(-0.0)).to_bits(), f32!(-0.0).to_bits());
	}

	#[test]
	fn sanity_check() {
		assert_eq!((f32!(-1.0)).round(), f32!(-1.0));
		assert_eq!((f32!(2.8)).round(), f32!(3.0));
		assert_eq!((f32!(-0.5)).round(), f32!(-1.0));
		assert_eq!((f32!(0.5)).round(), f32!(1.0));
		assert_eq!((f32!(-1.5)).round(), f32!(-2.0));
		assert_eq!((f32!(1.5)).round(), f32!(2.0));
	}
}
