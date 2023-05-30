// RAII-style guard guard for turning off floating point denormals
//
// previous code also had the following:
//        // All exceptions are masked
//        mxcsr |= ((1 << 6) - 1) << 7;
// but I don't know if this is actually necessary
//

// FTZ and DAZ
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
const X86_MASK: u32 = 0x8040;

// FTZ
#[cfg(target_arch = "aarch64")]
const AARCH64_MASK: u64 = 1 << 24;

pub struct DenormalGuard {
	#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
	mxcsr: u32,
	#[cfg(target_arch = "aarch64")]
	fpcr: u64,
}

impl DenormalGuard {
	fn new() -> Self {
		#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
		unsafe {
			#[cfg(all(target_arch = "x86_64", target_feature = "sse"))]
			use std::arch::x86_64::{_mm_getcsr, _mm_setcsr};

			#[cfg(all(target_arch = "x86", target_feature = "sse"))]
			use std::arch::x86::{_mm_getcsr, _mm_setcsr};

			let mxcsr = _mm_getcsr();
			_mm_setcsr(mxcsr | X86_MASK);
			DenormalGuard { mxcsr }
		}
		#[cfg(target_arch = "aarch64")]
		{
			let mut fpcr: u64;
			unsafe { std::arch::asm!("mrs {}, fpcr", out(reg) fpcr) };
			unsafe { std::arch::asm!("msr fpcr, {}", in(reg) fpcr | AARCH64_MASK) };

			DenormalGuard { fpcr }
		}
	}
}

impl Drop for DenormalGuard {
	fn drop(&mut self) {
		#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
		unsafe {
			#[cfg(all(target_arch = "x86_64", target_feature = "sse"))]
			use std::arch::x86_64::_mm_setcsr;

			#[cfg(all(target_arch = "x86", target_feature = "sse"))]
			use std::arch::x86::_mm_setcsr;

			_mm_setcsr(self.mxcsr);
		}
		#[cfg(target_arch = "aarch64")]
		{
			unsafe { std::arch::asm!("msr fpcr, {}", in(reg) self.fpcr) };
		}
	}
}

pub fn no_denormals<T, F: FnOnce() -> T>(func: F) -> T {
	let guard = DenormalGuard::new();
	let ret = func();
	std::mem::drop(guard);

	return ret;
}
