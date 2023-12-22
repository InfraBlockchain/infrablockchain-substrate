use crate::{soft_f32::F32, soft_f64::F64};

// Source: https://github.com/rust-lang/compiler-builtins/blob/3dea633a80d32da75e923a940d16ce98cce74822/src/float/extend.rs#L4
pub const fn extend(a: F32) -> F64 {
	let src_zero = 0;
	let src_one = 1;
	let src_bits = F32::BITS;
	let src_sign_bits = F32::SIGNIFICAND_BITS;
	let src_exp_bias = F32::EXPONENT_BIAS;
	let src_min_normal = F32::IMPLICIT_BIT;
	let src_infinity = F32::EXPONENT_MASK;
	let src_sign_mask = F32::SIGN_MASK;
	let src_abs_mask = src_sign_mask - src_one;
	let src_qnan = F32::SIGNIFICAND_MASK;
	let src_nan_code = src_qnan - src_one;

	let dst_bits = F64::BITS;
	let dst_sign_bits = F64::SIGNIFICAND_BITS;
	let dst_inf_exp = F64::EXPONENT_MAX;
	let dst_exp_bias = F64::EXPONENT_BIAS;
	let dst_min_normal = F64::IMPLICIT_BIT;

	let sign_bits_delta = dst_sign_bits - src_sign_bits;
	let exp_bias_delta = dst_exp_bias - src_exp_bias;
	let a_abs = a.repr() & src_abs_mask;
	let mut abs_result: u64 = 0;

	if a_abs.wrapping_sub(src_min_normal) < src_infinity.wrapping_sub(src_min_normal) {
		// a is a normal number.
		// Extend to the destination type by shifting the significand and
		// exponent into the proper position and rebiasing the exponent.
		let abs_dst = a_abs as u64;
		let bias_dst = exp_bias_delta as u64;
		abs_result = abs_dst.wrapping_shl(sign_bits_delta);
		abs_result += bias_dst.wrapping_shl(dst_sign_bits);
	} else if a_abs >= src_infinity {
		// a is NaN or infinity.
		// Conjure the result by beginning with infinity, then setting the qNaN
		// bit (if needed) and right-aligning the rest of the trailing NaN
		// payload field.
		let qnan_dst = (a_abs & src_qnan) as u64;
		let nan_code_dst = (a_abs & src_nan_code) as u64;
		let inf_exp_dst = dst_inf_exp as u64;
		abs_result = inf_exp_dst.wrapping_shl(dst_sign_bits);
		abs_result |= qnan_dst.wrapping_shl(sign_bits_delta);
		abs_result |= nan_code_dst.wrapping_shl(sign_bits_delta);
	} else if a_abs != src_zero {
		// a is denormal.
		// Renormalize the significand and clear the leading bit, then insert
		// the correct adjusted exponent in the destination type.
		let scale = a_abs.leading_zeros() - src_min_normal.leading_zeros();
		let abs_dst = a_abs as u64;
		let bias_dst = (exp_bias_delta - scale + 1) as u64;
		abs_result = abs_dst.wrapping_shl(sign_bits_delta + scale);
		abs_result = (abs_result ^ dst_min_normal) | (bias_dst.wrapping_shl(dst_sign_bits));
	}

	let sign_result = (a.repr() & src_sign_mask) as u64;
	F64::from_repr(abs_result | (sign_result.wrapping_shl(dst_bits - src_bits)))
}

// Source: https://github.com/rust-lang/compiler-builtins/blob/3dea633a80d32da75e923a940d16ce98cce74822/src/float/trunc.rs#L4
pub const fn trunc(a: F64) -> F32 {
	let src_zero = 0_u64;
	let src_one = 1_u64;
	let src_bits = F64::BITS;
	let src_exp_bias = F64::EXPONENT_BIAS;

	let src_min_normal = F64::IMPLICIT_BIT;
	let src_significand_mask = F64::SIGNIFICAND_MASK;
	let src_infinity = F64::EXPONENT_MASK;
	let src_sign_mask = F64::SIGN_MASK;
	let src_abs_mask = src_sign_mask - src_one;
	let round_mask = (src_one << (F64::SIGNIFICAND_BITS - F32::SIGNIFICAND_BITS)) - src_one;
	let halfway = src_one << (F64::SIGNIFICAND_BITS - F32::SIGNIFICAND_BITS - 1);
	let src_qnan = src_one << (F64::SIGNIFICAND_BITS - 1);
	let src_nan_code = src_qnan - src_one;

	let dst_zero = 0_u32;
	let dst_one = 1_u32;
	let dst_bits = F32::BITS;
	let dst_inf_exp = F32::EXPONENT_MAX;
	let dst_exp_bias = F32::EXPONENT_BIAS;

	let underflow_exponent: u64 = (src_exp_bias + 1 - dst_exp_bias) as u64;
	let overflow_exponent: u64 = (src_exp_bias + dst_inf_exp - dst_exp_bias) as u64;
	let underflow: u64 = underflow_exponent << F64::SIGNIFICAND_BITS;
	let overflow: u64 = overflow_exponent << F64::SIGNIFICAND_BITS;

	let dst_qnan = 1_u32 << (F32::SIGNIFICAND_BITS - 1);
	let dst_nan_code = dst_qnan - dst_one;

	let sign_bits_delta = F64::SIGNIFICAND_BITS - F32::SIGNIFICAND_BITS;
	// Break a into a sign and representation of the absolute value.
	let a_abs = a.repr() & src_abs_mask;
	let sign = a.repr() & src_sign_mask;
	let mut abs_result: u32;

	if a_abs.wrapping_sub(underflow) < a_abs.wrapping_sub(overflow) {
		// The exponent of a is within the range of normal numbers in the
		// destination format.  We can convert by simply right-shifting with
		// rounding and adjusting the exponent.
		abs_result = (a_abs >> sign_bits_delta) as u32;
		let tmp = src_exp_bias.wrapping_sub(dst_exp_bias) << F32::SIGNIFICAND_BITS;
		abs_result = abs_result.wrapping_sub(tmp as u32);

		let round_bits = a_abs & round_mask;
		if round_bits > halfway {
			// Round to nearest.
			abs_result += dst_one;
		} else if round_bits == halfway {
			// Tie to even.
			abs_result += abs_result & dst_one;
		};
	} else if a_abs > src_infinity {
		// a is NaN.
		// Conjure the result by beginning with infinity, setting the qNaN
		// bit and inserting the (truncated) trailing NaN field.
		abs_result = (dst_inf_exp << F32::SIGNIFICAND_BITS) as u32;
		abs_result |= dst_qnan;
		abs_result |= dst_nan_code &
			((a_abs & src_nan_code) >> (F64::SIGNIFICAND_BITS - F32::SIGNIFICAND_BITS)) as u32;
	} else if a_abs >= overflow {
		// a overflows to infinity.
		abs_result = (dst_inf_exp << F32::SIGNIFICAND_BITS) as u32;
	} else {
		// a underflows on conversion to the destination type or is an exact
		// zero.  The result may be a denormal or zero.  Extract the exponent
		// to get the shift amount for the denormalization.
		let a_exp: u32 = (a_abs >> F64::SIGNIFICAND_BITS) as u32;
		let shift = src_exp_bias - dst_exp_bias - a_exp + 1;

		let significand = (a.repr() & src_significand_mask) | src_min_normal;

		// Right shift by the denormalization amount with sticky.
		if shift > F64::SIGNIFICAND_BITS {
			abs_result = dst_zero;
		} else {
			let sticky =
				if (significand << (src_bits - shift)) != src_zero { src_one } else { src_zero };
			let denormalized_significand: u64 = significand >> shift | sticky;
			abs_result = (denormalized_significand >>
				(F64::SIGNIFICAND_BITS - F32::SIGNIFICAND_BITS)) as u32;
			let round_bits = denormalized_significand & round_mask;
			// Round to nearest
			if round_bits > halfway {
				abs_result += dst_one;
			}
			// Ties to even
			else if round_bits == halfway {
				abs_result += abs_result & dst_one;
			};
		}
	}

	// Apply the signbit to the absolute value.
	F32::from_repr(abs_result | sign.wrapping_shr(src_bits - dst_bits) as u32)
}

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

// Source: https://github.com/rust-lang/compiler-builtins/blob/3dea633a80d32da75e923a940d16ce98cce74822/src/float/conv.rs#L8C1-L18C6
const fn u32_to_f32_bits(i: u32) -> u32 {
	if i == 0 {
		return 0;
	}
	let n = i.leading_zeros();
	let a = (i << n) >> 8; // Significant bits, with bit 24 still in tact.
	let b = (i << n) << 24; // Insignificant bits, only relevant for rounding.
	let m = a + ((b - (b >> 31 & !a)) >> 31); // Add one when we need to round up. Break ties to even.
	let e = 157 - n; // Exponent plus 127, minus one.
	(e << 23) + m // + not |, so the mantissa can overflow into the exponent.
}

pub const fn u32_to_f32(i: u32) -> F32 {
	F32::from_bits(u32_to_f32_bits(i))
}

// Source: https://github.com/rust-lang/compiler-builtins/blob/3dea633a80d32da75e923a940d16ce98cce74822/src/float/conv.rs#L148C1-L161C6
pub const fn f32_to_u32(f: F32) -> u32 {
	let fbits = f.to_bits();
	if fbits < 127 << 23 {
		// >= 0, < 1
		0
	} else if fbits < 159 << 23 {
		// >= 1, < max
		let m = 1 << 31 | fbits << 8; // Mantissa and the implicit 1-bit.
		let s = 158 - (fbits >> 23); // Shift based on the exponent and bias.
		m >> s
	} else if fbits <= 255 << 23 {
		// >= max (incl. inf)
		u32::MAX
	} else {
		// Negative or NaN
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
