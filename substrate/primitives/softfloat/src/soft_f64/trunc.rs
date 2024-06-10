use super::F64;

pub(crate) const fn trunc(x: F64) -> F64 {
	let mut i: u64 = x.to_bits();
	let mut e: i64 = (i >> 52 & 0x7ff) as i64 - 0x3ff + 12;

	if e >= 52 + 12 {
		return x
	}
	if e < 12 {
		e = 1;
	}
	let m = -1i64 as u64 >> e;
	if (i & m) == 0 {
		return x
	}
	i &= !m;
	F64::from_bits(i)
}

#[cfg(test)]
mod tests {
	#[test]
	fn sanity_check() {
		assert_eq!(super::trunc(f64!(1.1)), f64!(1.0));
	}
}
