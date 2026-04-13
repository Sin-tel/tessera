// TODO: Probably want a nicer error enum instead of String

use vst3::Steinberg::{
	kInternalError, kInvalidArgument, kNoInterface, kNotImplemented, kNotInitialized, kOutOfMemory,
	kResultFalse, kResultOk, tresult,
};

pub trait ToResultExt {
	fn as_result(&self) -> Result<(), String>;
}

impl ToResultExt for tresult {
	fn as_result(&self) -> Result<(), String> {
		#[allow(non_upper_case_globals)]
		match *self {
			kResultOk => Ok(()),
			kResultFalse => Err("Operation failed".into()),
			kInvalidArgument => Err("Invalid argument".into()),
			kNotImplemented => Err("Not implemented".into()),
			kInternalError => Err("Internal error".into()),
			kOutOfMemory => Err("Out of memory".into()),
			kNoInterface => Err("No interface".into()),
			kNotInitialized => Err("Not initialized".into()),
			other => Err(format!("Unknown error code ({other})")),
		}
	}
}
