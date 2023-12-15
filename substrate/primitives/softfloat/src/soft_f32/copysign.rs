use super::F32;

/// Sign of Y, magnitude of X (F32)
///
/// Constructs a number with the magnitude (absolute value) of its
/// first argument, `x`, and the sign of its second argument, `y`.
pub(crate) const fn copysign(x: F32, y: F32) -> F32 {
	let mut ux = x.to_bits();
	let uy = y.to_bits();
	ux &= 0x7fffffff;
	ux |= uy & 0x80000000;
	F32::from_bits(ux)
}

#[cfg(test)]
mod test {
	#[test]
	fn sanity_check() {
		assert_eq!(f32!(1.0).copysign(f32!(-0.0)), f32!(-1.0))
	}
}
