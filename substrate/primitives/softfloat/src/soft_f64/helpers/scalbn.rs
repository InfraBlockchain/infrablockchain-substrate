use crate::soft_f64::F64;

pub(crate) const fn scalbn(x: F64, mut n: i32) -> F64 {
	let x1p1023 = F64::from_bits(0x7fe0000000000000); // 0x1p1023 === 2 ^ 1023
	let x1p53 = F64::from_bits(0x4340000000000000); // 0x1p53 === 2 ^ 53
	let x1p_1022 = F64::from_bits(0x0010000000000000); // 0x1p-1022 === 2 ^ (-1022)

	let mut y = x;

	if n > 1023 {
		y = y.mul(x1p1023);
		n -= 1023;
		if n > 1023 {
			y = y.mul(x1p1023);
			n -= 1023;
			if n > 1023 {
				n = 1023;
			}
		}
	} else if n < -1022 {
		/* make sure final n < -53 to avoid double
		rounding in the subnormal range */
		y = y.mul(x1p_1022.mul(x1p53));
		n += 1022 - 53;
		if n < -1022 {
			y = y.mul(x1p_1022.mul(x1p53));
			n += 1022 - 53;
			if n < -1022 {
				n = -1022;
			}
		}
	}
	y.mul(F64::from_bits(((0x3ff + n) as u64) << 52))
}
