use std::fs::File;
use std::fs::OpenOptions;
use std::io::LineWriter;

pub fn init_logging() {
	use simplelog::*;
	log_panics::init();

	let config = ConfigBuilder::new()
		.set_time_level(LevelFilter::Off)
		.set_location_level(LevelFilter::Off)
		.set_target_level(LevelFilter::Off)
		.build();

	let filename = "../out/out.log";

	// create empty new file
	File::create(filename).unwrap();
	// append mode for atomic writes
	let f = OpenOptions::new().append(true).open(filename).unwrap();
	// buffer lines
	let f_write = LineWriter::new(f);

	CombinedLogger::init(vec![
		SimpleLogger::new(LevelFilter::Info, config.clone()),
		WriteLogger::new(LevelFilter::Trace, config, f_write),
	])
	.unwrap();
}

macro_rules! log_trace {
    ($($t:tt)*) => {{
        use assert_no_alloc::permit_alloc;
        use log::trace;
        permit_alloc(|| {
            trace!($($t)*)
        })
    }};
}

macro_rules! log_info {
    ($($t:tt)*) => {{
        use assert_no_alloc::permit_alloc;
        use log::info;
        permit_alloc(|| {
            info!($($t)*)
        })
    }};
}

macro_rules! log_debug {
    ($($t:tt)*) => {{
        use assert_no_alloc::permit_alloc;
        use log::debug;
        permit_alloc(|| {
            debug!($($t)*)
        })
    }};
}

macro_rules! log_warn {
    ($($t:tt)*) => {{
        use assert_no_alloc::permit_alloc;
        use log::warn;
        permit_alloc(|| {
            warn!($($t)*)
        })
    }};
}

macro_rules! log_error {
    ($($t:tt)*) => {{
        use assert_no_alloc::permit_alloc;
        use log::error;
        permit_alloc(|| {
            error!($($t)*)
        })
    }};
}

pub(crate) use log_debug;
pub(crate) use log_error;
pub(crate) use log_info;
pub(crate) use log_trace;
pub(crate) use log_warn;
