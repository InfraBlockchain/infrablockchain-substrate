use crate::soft_f64::F64;
use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

mod helpers;

pub mod add;
pub mod cmp;
pub mod copysign;
pub mod cos;
pub mod div;
pub mod floor;
pub mod mul;
pub mod pow;
pub mod round;
pub mod sin;
pub mod sqrt;
pub mod trunc;

#[derive(Default, Copy, Clone, Debug, Decode, Encode, TypeInfo, MaxEncodedLen)]
#[repr(transparent)]
struct Bits32(u32);

/// A pure software implementation of `f32`.
#[derive(Default, Copy, Clone, Debug, Decode, Encode, TypeInfo, MaxEncodedLen)]
#[repr(transparent)]
pub struct F32(Bits32);

impl F32 {
	pub const fn from_native_f32(a: f32) -> Self {
		Self(unsafe { core::mem::transmute(a) })
	}

	pub const fn to_native_f32(self) -> f32 {
		unsafe { core::mem::transmute(self.0) }
	}

	pub const fn to_f64(self) -> F64 {
		crate::conv::extend(self)
	}

	pub const fn from_f64(a: F64) -> Self {
		crate::conv::trunc(a)
	}

	pub const fn to_u32(self) -> u32 {
		crate::conv::f32_to_u32(self)
	}

	pub const fn from_u32(a: u32) -> Self {
		crate::conv::u32_to_f32(a)
	}

	pub const fn from_bits(a: u32) -> Self {
		Self(Bits32(a))
	}

	pub const fn to_bits(self) -> u32 {
		self.0 .0
	}

	pub const fn add(self, rhs: Self) -> Self {
		add::add(self, rhs)
	}

	pub const fn mul(self, rhs: Self) -> Self {
		mul::mul(self, rhs)
	}

	pub const fn div(self, rhs: Self) -> Self {
		div::div(self, rhs)
	}

	pub const fn cmp(&self, rhs: &Self) -> Option<core::cmp::Ordering> {
		cmp::cmp(self, rhs)
	}

	pub const fn neg(self) -> Self {
		Self::from_repr(self.repr() ^ Self::SIGN_MASK)
	}

	pub const fn sub(self, rhs: Self) -> Self {
		self.add(rhs.neg())
	}

	pub const fn sqrt(self) -> Self {
		sqrt::sqrtf(self)
	}

	pub const fn powi(self, n: i32) -> Self {
		pow::pow(self, n)
	}

	pub const fn copysign(self, other: Self) -> Self {
		copysign::copysign(self, other)
	}

	pub const fn trunc(self) -> Self {
		trunc::trunc(self)
	}

	pub const fn round(self) -> Self {
		round::round(self)
	}

	pub const fn floor(self) -> Self {
		floor::floor(self)
	}

	pub const fn sin(self) -> Self {
		sin::sinf(self)
	}

	pub const fn cos(self) -> Self {
		cos::cos(self)
	}
}

type SelfInt = u32;
type SelfSignedInt = i32;
type SelfExpInt = i16;

#[allow(unused)]
impl F32 {
	const ZERO: Self = f32!(0.0);
	const ONE: Self = f32!(1.0);
	pub(crate) const BITS: u32 = 32;
	pub(crate) const SIGNIFICAND_BITS: u32 = 23;
	pub(crate) const EXPONENT_BITS: u32 = Self::BITS - Self::SIGNIFICAND_BITS - 1;
	pub(crate) const EXPONENT_MAX: u32 = (1 << Self::EXPONENT_BITS) - 1;
	pub(crate) const EXPONENT_BIAS: u32 = Self::EXPONENT_MAX >> 1;
	pub(crate) const SIGN_MASK: SelfInt = 1 << (Self::BITS - 1);
	pub(crate) const SIGNIFICAND_MASK: SelfInt = (1 << Self::SIGNIFICAND_BITS) - 1;
	pub(crate) const IMPLICIT_BIT: SelfInt = 1 << Self::SIGNIFICAND_BITS;
	pub(crate) const EXPONENT_MASK: SelfInt = !(Self::SIGN_MASK | Self::SIGNIFICAND_MASK);

	pub(crate) const fn repr(self) -> SelfInt {
		self.to_bits()
	}
	const fn signed_repr(self) -> SelfSignedInt {
		self.to_bits() as SelfSignedInt
	}
	const fn sign(self) -> bool {
		self.signed_repr() < 0
	}
	const fn exp(self) -> SelfExpInt {
		((self.to_bits() & Self::EXPONENT_MASK) >> Self::SIGNIFICAND_BITS) as SelfExpInt
	}
	const fn frac(self) -> SelfInt {
		self.to_bits() & Self::SIGNIFICAND_MASK
	}
	const fn imp_frac(self) -> SelfInt {
		self.frac() | Self::IMPLICIT_BIT
	}
	pub(crate) const fn from_repr(a: SelfInt) -> Self {
		Self::from_bits(a)
	}
	const fn from_parts(sign: bool, exponent: SelfInt, significand: SelfInt) -> Self {
		Self::from_repr(
			((sign as SelfInt) << (Self::BITS - 1)) |
				((exponent << Self::SIGNIFICAND_BITS) & Self::EXPONENT_MASK) |
				(significand & Self::SIGNIFICAND_MASK),
		)
	}
	const fn normalize(significand: SelfInt) -> (i32, SelfInt) {
		let shift = significand
			.leading_zeros()
			.wrapping_sub((1u32 << Self::SIGNIFICAND_BITS).leading_zeros());
		(1i32.wrapping_sub(shift as i32), significand << shift as SelfInt)
	}
	const fn is_subnormal(self) -> bool {
		(self.repr() & Self::EXPONENT_MASK) == 0
	}
}

const fn u64_lo(x: u64) -> u32 {
	x as u32
}

const fn u64_hi(x: u64) -> u32 {
	(x >> 32) as u32
}

const fn u32_widen_mul(a: u32, b: u32) -> (u32, u32) {
	let x = u64::wrapping_mul(a as _, b as _);
	(u64_lo(x), u64_hi(x))
}

#[test]
fn test_conversion_f32_to_and_from_u32() {
	assert_eq!(F32::from_native_f32(1234.0).to_u32(), 1234);
	assert_eq!(F32::from_u32(1234), F32::from_native_f32(1234.0));
}
