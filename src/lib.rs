pub mod benchmarks;
pub mod lptau;
pub mod optimizers;

#[cfg(feature = "fortran")]
pub mod fortran;

#[cfg(feature = "python")]
pub mod python_bindings;

pub use benchmarks::{fubran, fugold, fuhar3, fuhar6, furasn, fush10, fush5, fush7};
#[cfg(feature = "fortran")]
pub use fortran::{
	fortran_bayes1, fortran_exkor, fortran_extr, fortran_flexi, fortran_glopt, fortran_lbayes,
	fortran_lpmin, fortran_lptau, fortran_mig1, fortran_mig2, fortran_mivar4, fortran_reqp,
	fortran_unt, FortranObjective,
};
pub use lptau::{lp_tau_point, AtsGenerator};
pub use optimizers::{
	anal1, anal2, bayes1, exkor, extr, flexi, glopt, lbayes, lpmin, mig1, mig2, mivar4, reqp,
	unt, AnalResult, Bayes1Config, ExkorConfig, ExtrConfig, FlexiConfig, GloptConfig,
	LbayesConfig, LpminConfig, Mig1Config, Mig2Config, Mivar4Config, OptError, OptResult,
	ReqpConfig, UntConfig,
};

#[no_mangle]
pub extern "C" fn globalopt_furasn_c(x: *const f64, n: *const i32, out: *mut f64) {
	if out.is_null() {
		return;
	}
	if x.is_null() || n.is_null() {
		// SAFETY: out was checked for null above and points to caller-owned memory.
		unsafe {
			*out = f64::NAN;
		}
		return;
	}

	// SAFETY: `n` and `x` pointers come from caller and are validated above.
	let len = unsafe { *n };
	if len <= 0 {
		// SAFETY: out was checked for null above and points to caller-owned memory.
		unsafe {
			*out = 0.0;
		}
		return;
	}

	// SAFETY: caller provides a valid `x` buffer of length `len` doubles.
	let xs = unsafe { std::slice::from_raw_parts(x, len as usize) };
	let value = benchmarks::furasn(xs);
	// SAFETY: out was checked for null above and points to caller-owned memory.
	unsafe {
		*out = value;
	}
}
