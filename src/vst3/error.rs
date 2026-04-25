use std::fmt;
use vst3::Steinberg::{
	kInternalError, kInvalidArgument, kNoInterface, kNotImplemented, kNotInitialized, kOutOfMemory,
	kResultFalse, kResultOk, tresult,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vst3Error {
	False,
	InvalidArgument,
	NotImplemented,
	InternalError,
	OutOfMemory,
	NoInterface,
	NotInitialized,
	Unknown(i32),
}

impl fmt::Display for Vst3Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Vst3Error::False => write!(f, "Operation failed (kResultFalse)"),
			Vst3Error::InvalidArgument => write!(f, "Invalid argument (kInvalidArgument)"),
			Vst3Error::NotImplemented => write!(f, "Not implemented (kNotImplemented)"),
			Vst3Error::InternalError => write!(f, "Internal error (kInternalError)"),
			Vst3Error::OutOfMemory => write!(f, "Out of memory (kOutOfMemory)"),
			Vst3Error::NoInterface => write!(f, "No interface (kNoInterface)"),
			Vst3Error::NotInitialized => write!(f, "Not initialized (kNotInitialized)"),
			Vst3Error::Unknown(code) => write!(f, "Unknown VST3 error code ({code})"),
		}
	}
}

// Implement the standard Error trait so anyhow can use it
impl std::error::Error for Vst3Error {}

pub trait ToResultExt {
	fn as_result(&self) -> Result<(), Vst3Error>;
}

impl ToResultExt for tresult {
	fn as_result(&self) -> Result<(), Vst3Error> {
		#[allow(non_upper_case_globals)]
		match *self {
			kResultOk => Ok(()),
			kResultFalse => Err(Vst3Error::False),
			kInvalidArgument => Err(Vst3Error::InvalidArgument),
			kNotImplemented => Err(Vst3Error::NotImplemented),
			kInternalError => Err(Vst3Error::InternalError),
			kOutOfMemory => Err(Vst3Error::OutOfMemory),
			kNoInterface => Err(Vst3Error::NoInterface),
			kNotInitialized => Err(Vst3Error::NotInitialized),
			other => Err(Vst3Error::Unknown(other)),
		}
	}
}
