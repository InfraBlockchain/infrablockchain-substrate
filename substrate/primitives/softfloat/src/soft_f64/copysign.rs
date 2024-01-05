use super::F64;

/// Sign of Y, magnitude of X (f64)
///
/// Constructs a number with the magnitude (absolute value) of its
/// first argument, `x`, and the sign of its second argument, `y`.
pub(crate) const fn copysign(x: F64, y: F64) -> F64 {
	let mut ux = x.to_bits();
	let uy = y.to_bits();
	ux &= (!0) >> 1;
	ux |= uy & (1 << 63);
	F64::from_bits(ux)
}
