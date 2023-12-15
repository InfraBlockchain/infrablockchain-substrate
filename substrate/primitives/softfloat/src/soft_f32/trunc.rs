use super::F32;

pub(crate) const fn trunc(x: F32) -> F32 {
	let mut i: u32 = x.to_bits();
	let mut e: i32 = (i >> 23 & 0xff) as i32 - 0x7f + 9;
	let m: u32;

	if e >= 23 + 9 {
		return x;
	}
	if e < 9 {
		e = 1;
	}
	m = -1i32 as u32 >> e;
	if (i & m) == 0 {
		return x;
	}
	i &= !m;
	F32::from_bits(i)
}

#[cfg(test)]
mod tests {
	#[test]
	fn sanity_check() {
		assert_eq!(super::trunc(f32!(1.1)), f32!(1.0));
	}
}
