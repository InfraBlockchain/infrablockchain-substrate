use crate::soft_f64::F64;

// Source: https://github.com/rust-lang/compiler-builtins/blob/3dea633a80d32da75e923a940d16ce98cce74822/src/float/conv.rs#L20C1-L28C6
pub const fn u32_to_f64_bits(i: u32) -> u64 {
	if i == 0 {
		return 0;
	}
	let n = i.leading_zeros();
	let m = (i as u64) << (21 + n); // Significant bits, with bit 53 still in tact.
	let e = 1053 - n as u64; // Exponent plus 1023, minus one.
	(e << 52) + m // Bit 53 of m will overflow into e.
}

// Source: https://github.com/rust-lang/compiler-builtins/blob/3dea633a80d32da75e923a940d16ce98cce74822/src/float/conv.rs#L115
pub const fn i32_to_f64(i: i32) -> F64 {
	let sign_bit = ((i >> 31) as u64) << 63;
	F64::from_bits(u32_to_f64_bits(i.unsigned_abs()) | sign_bit)
}

// Source: https://github.com/rust-lang/compiler-builtins/blob/3dea633a80d32da75e923a940d16ce98cce74822/src/float/conv.rs#L298
pub const fn f64_to_i32(f: F64) -> i32 {
	let fbits = f.to_bits() & !0 >> 1; // Remove sign bit.
	if fbits < 1023 << 52 {
		// >= 0, < 1
		0
	} else if fbits < 1054 << 52 {
		// >= 1, < max
		let m = 1 << 31 | (fbits >> 21) as u32; // Mantissa and the implicit 1-bit.
		let s = 1054 - (fbits >> 52); // Shift based on the exponent and bias.
		let u = (m >> s) as i32; // Unsigned result.
		if f.is_sign_negative() {
			-u
		} else {
			u
		}
	} else if fbits <= 2047 << 52 {
		// >= max (incl. inf)
		if f.is_sign_negative() {
			i32::MIN
		} else {
			i32::MAX
		}
	} else {
		// NaN
		0
	}
}

pub const fn u128_to_f64_bits(i: u128) -> u64 {
	if i == 0 {
		return 0;
	}

	let n = i.leading_zeros();
	let y = i.wrapping_shl(n);
	let a = (y >> 75) as u64; // Significant bits, with bit 53 still in tact.
	let b = (y >> 11 | y & 0xFFFF_FFFF) as u64; // Insignificant bits, only relevant for rounding.
	let m = a + ((b - (b >> 63 & !a)) >> 63); // Add one when we need to round up. Break ties to even.
	let e = if i == 0 { 0 } else { 1149 - n as u64 }; // Exponent plus 1023, minus one, except for zero.
	(e << 52) + m // + not |, so the mantissa can overflow into the exponent.
}

pub const fn i128_to_f64(i: i128) -> F64 {
	let sign_bit = ((i >> 127) as u64) << 63;
	// Simplified conversion: i128 to u128 and then convert the lower 64 bits
	let truncated = if i == i128::MIN {
		// The absolute value of i128::MIN is the same as i128::MAX + 1. In this case, use
		// u128::MAX.
		u128::MAX
	} else {
		i.abs() as u128
	};
	let f64_bits = u128_to_f64_bits(truncated);
	F64::from_bits(f64_bits | sign_bit)
}

pub const fn f64_to_i128(f: F64) -> i128 {
	let fbits = f.to_bits() & !0 >> 1; // Remove sign bit.

	// Check if the value is within the range of i128
	if fbits < 1023 << 52 {
		// >= 0, < 1
		0
	} else if fbits < (1023 + 127) << 52 {
		// >= 1, < max i128
		let m = 1 << 127 | (fbits as u128) << 75; // Mantissa and the implicit 1-bit.
		let s = (1023 + 127) - (fbits >> 52); // Shift based on the exponent and bias.
		let u = (m >> s) as i128; // Unsigned result.
		if f.is_sign_negative() {
			-u
		} else {
			u
		}
	} else if fbits <= 2047 << 52 {
		// >= max i128 (incl. inf)
		if f.is_sign_negative() {
			i128::MIN
		} else {
			i128::MAX
		}
	} else {
		// it_is_nan();
		// NaN
		0
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_F64_u128_conversion_i128_to_f64() {
		assert_eq!(i128_to_f64(i128::MAX).to_bits(), F64::from_i128(i128::MAX).to_bits());
		assert_eq!(i128_to_f64(i128::MIN).to_bits(), F64::from_i128(i128::MIN).to_bits());
		assert_eq!(i128_to_f64(0).to_bits(), F64::from_i128(0).to_bits());

		assert_eq!(i128_to_f64(12345).to_bits(), F64::from_i128(12345).to_bits());
		assert_eq!(i128_to_f64(-12345).to_bits(), F64::from_i128(-12345).to_bits());
	}

	#[test]
	fn test_F64_u128_conversion_f64_to_i128() {
		assert_eq!(f64_to_i128(F64::from(f64::MAX)), i128::MAX);
		assert_eq!(f64_to_i128(F64::from(f64::MIN)), i128::MIN);
		assert_eq!(f64_to_i128(F64::from(0.0)), 0);
		assert_eq!(f64_to_i128(F64::from(f64::NAN)), 0);
		assert_eq!(f64_to_i128(F64::from(f64::INFINITY)), i128::MAX);
		assert_eq!(f64_to_i128(F64::from(f64::NEG_INFINITY)), i128::MIN);

		assert_eq!(f64_to_i128(F64::from(123.456)), 123);
		assert_eq!(f64_to_i128(F64::from(-123.456)), -123);
	}
}
