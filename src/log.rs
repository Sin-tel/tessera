use crate::context::LuaMessage;
use simplelog::Config;
use simplelog::LevelFilter;
use simplelog::SharedLogger;
use standard_log::{Level, Log, Metadata, Record};
use std::fs::File;
use std::fs::OpenOptions;
use std::io::LineWriter;
use std::sync::mpsc::SyncSender;

pub fn init_logging(lua_tx: SyncSender<LuaMessage>) {
	use simplelog::*;

	let config = ConfigBuilder::new()
		.set_time_level(LevelFilter::Off)
		.set_location_level(LevelFilter::Off)
		.set_target_level(LevelFilter::Off)
		.build();

	let filename = "out/out.log";

	// create empty new file
	File::create(filename).unwrap();

	// append mode for atomic writes
	let f = OpenOptions::new().append(true).open(filename).unwrap();

	// buffer lines
	let f_write = LineWriter::new(f);

	CombinedLogger::init(vec![
		SimpleLogger::new(LevelFilter::Info, config.clone()),
		WriteLogger::new(LevelFilter::Info, config, f_write),
		GuiLogger::new(LevelFilter::Info, lua_tx),
	])
	.unwrap();
	// Has to go last otherwise previous errors don't work
	log_panics::init();
}

pub struct GuiLogger {
	sender: SyncSender<LuaMessage>,
	level_filter: LevelFilter,
}

impl GuiLogger {
	pub fn new(level_filter: LevelFilter, sender: SyncSender<LuaMessage>) -> Box<Self> {
		Box::new(Self { level_filter, sender })
	}
}

impl Log for GuiLogger {
	fn enabled(&self, metadata: &Metadata) -> bool {
		// Only send Info or higher to the GUI to prevent flooding
		metadata.level() <= self.level_filter
	}

	fn log(&self, record: &Record) {
		if self.enabled(record.metadata()) {
			let msg = LuaMessage::Log {
				level: record.level().to_string(),
				message: format!("{}", record.args()),
			};
			// Send asynchronously. If the queue is full/disconnected,
			// we silently ignore to prevent panicking the audio/worker thread.
			let _ = self.sender.send(msg);
		}
	}

	fn flush(&self) {}
}

impl SharedLogger for GuiLogger {
	fn level(&self) -> LevelFilter {
		self.level_filter
	}
	fn config(&self) -> Option<&Config> {
		None
	}
	fn as_log(self: Box<Self>) -> Box<dyn Log> {
		Box::new(self)
	}
}

#[macro_export]
macro_rules! log_trace {
    ($($t:tt)*) => {{
        use assert_no_alloc::permit_alloc;
        permit_alloc(|| {
            standard_log::trace!($($t)*)
        })
    }};
}

#[macro_export]
macro_rules! log_info {
    ($($t:tt)*) => {{
        use assert_no_alloc::permit_alloc;
        permit_alloc(|| {
            standard_log::info!($($t)*)
        })
    }};
}

#[macro_export]
macro_rules! log_debug {
    ($($t:tt)*) => {{
        use assert_no_alloc::permit_alloc;
        permit_alloc(|| {
            standard_log::debug!($($t)*)
        })
    }};
}

#[macro_export]
macro_rules! log_warn {
    ($($t:tt)*) => {{
        use assert_no_alloc::permit_alloc;
        permit_alloc(|| {
            standard_log::warn!($($t)*)
        })
    }};
}

#[macro_export]
macro_rules! log_error {
    ($($t:tt)*) => {{
        use assert_no_alloc::permit_alloc;
        permit_alloc(|| {
            standard_log::error!($($t)*)
        })
    }};
}

pub use log_debug;
pub use log_error;
pub use log_info;
pub use log_trace;
pub use log_warn;
