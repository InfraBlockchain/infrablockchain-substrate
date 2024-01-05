// origin: FreeBSD /usr/src/lib/msun/src/s_cos.c */,
// https://github.com/rust-lang/libm/blob/4c8a973741c014b11ce7f1477693a3e5d4ef9609/src/math/cos.rs
//
// ====================================================
// Copyright (C) 1993 by Sun Microsystems, Inc. All rights reserved.
//
// Developed at SunPro, a Sun Microsystems, Inc. business.
// Permission to use, copy, modify, and distribute this
// software is freely granted, provided that this notice
// is preserved.
// ====================================================

use super::{
	helpers::{k_cos, k_sin, rem_pio2},
	F64,
};

// cos(x)
// Return cosine function of x.
//
// kernel function:
//      k_sin           ... sine function on [-pi/4,pi/4]
//      k_cos           ... cosine function on [-pi/4,pi/4]
//      rem_pio2        ... argument reduction routine
//
// Method.
//      Let S,C and T denote the sin, cos and tan respectively on
//      [-PI/4, +PI/4]. Reduce the argument x to y1+y2 = x-k*pi/2
//      in [-pi/4 , +pi/4], and let n = k mod 4.
//      We have
//
//          n        sin(x)      cos(x)        tan(x)
//     ----------------------------------------------------------
//          0          S           C             T
//          1          C          -S            -1/T
//          2         -S          -C             T
//          3         -C           S            -1/T
//     ----------------------------------------------------------
//
// Special cases:
//      Let trig be any of sin, cos, or tan.
//      trig(+-INF)  is NaN, with signals;
//      trig(NaN)    is that NaN;
//
// Accuracy:
//      TRIG(x) returns trig(x) nearly rounded
//
pub(crate) const fn cos(x: F64) -> F64 {
	let ix = (F64::to_bits(x) >> 32) as u32 & 0x7fffffff;

	/* |x| ~< pi/4 */
	if ix <= 0x3fe921fb {
		if ix < 0x3e46a09e {
			/* if x < 2**-27 * sqrt(2) */
			/* raise inexact if x != 0 */
			if x.to_i32() == 0 {
				return F64::ONE
			}
		}
		return k_cos(x, F64::ZERO)
	}

	/* cos(Inf or NaN) is NaN */
	if ix >= 0x7ff00000 {
		return x.sub(x)
	}

	/* argument reduction needed */
	let (n, y0, y1) = rem_pio2(x);
	match n & 3 {
		0 => k_cos(y0, y1),
		1 => k_sin(y0, y1, 1).neg(),
		2 => k_cos(y0, y1).neg(),
		_ => k_sin(y0, y1, 1),
	}
}

#[cfg(test)]
mod test {
	#[test]
	fn test_large_neg() {
		assert_eq!(f64!(-1647101.0).cos().to_native_f64(), (-1647101.0_f64).cos())
	}
}
