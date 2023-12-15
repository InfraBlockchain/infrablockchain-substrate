/* origin: FreeBSD /usr/src/lib/msun/src/k_cosf.c */
/*
 * Conversion to float by Ian Lance Taylor, Cygnus Support, ian@cygnus.com.
 * Debugged and optimized by Bruce D. Evans.
 */
/*
 * ====================================================
 * Copyright (C) 1993 by Sun Microsystems, Inc. All rights reserved.
 *
 * Developed at SunPro, a Sun Microsystems, Inc. business.
 * Permission to use, copy, modify, and distribute this
 * software is freely granted, provided that this notice
 * is preserved.
 * ====================================================
 */

use crate::{soft_f32::F32, soft_f64::F64};

/* |cos(x) - c(x)| < 2**-34.1 (~[-5.37e-11, 5.295e-11]). */
const C0: F64 = f64!(-0.499999997251031003120); /* -0x1ffffffd0c5e81.0p-54 */
const C1: F64 = f64!(0.0416666233237390631894); /* 0x155553e1053a42.0p-57 */
const C2: F64 = f64!(-0.00138867637746099294692); /* -0x16c087e80f1e27.0p-62 */
const C3: F64 = f64!(0.0000243904487962774090654); /* 0x199342e0ee5069.0p-68 */

#[cfg_attr(all(test, assert_no_panic), no_panic::no_panic)]
pub(crate) const fn k_cosf(x: F64) -> F32 {
	let z = x.mul(x);
	let w = z.mul(z);
	let r = C2.add(z.mul(C3));
	(((f64!(1.0).add(z.mul(C0))).add(w.mul(C1))).add((w.mul(z)).mul(r))).to_f32()
}
