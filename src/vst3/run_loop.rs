// Linux::IRunLoop: on Linux, plugin GUIs have no event loop of their own. They register file
// descriptors and timers with the host, which calls them back on the main thread.
// Exposed through PluginFrame, polled from the main loop (Vst3Editor::poll_run_loop).

use std::sync::Mutex;
use std::time::{Duration, Instant};
use vst3::ComPtr;
use vst3::Steinberg::Linux::{
	FileDescriptor, IEventHandler, IEventHandlerTrait, ITimerHandler, ITimerHandlerTrait,
	TimerInterval,
};
use vst3::Steinberg::{kInvalidArgument, kResultOk, tresult};
use vst3::com_scrape_types::ComRef;

struct Timer {
	handler: ComPtr<ITimerHandler>,
	interval: Duration,
	next: Instant,
}

#[derive(Default)]
pub struct RunLoop {
	handlers: Mutex<Vec<(ComPtr<IEventHandler>, FileDescriptor)>>,
	timers: Mutex<Vec<Timer>>,
}

impl RunLoop {
	pub fn register_event_handler(
		&self,
		handler: *mut IEventHandler,
		fd: FileDescriptor,
	) -> tresult {
		let Some(handler) = (unsafe { ComRef::from_raw(handler) }) else {
			return kInvalidArgument;
		};
		self.handlers.lock().unwrap().push((handler.to_com_ptr(), fd));
		kResultOk
	}

	pub fn unregister_event_handler(&self, handler: *mut IEventHandler) -> tresult {
		self.handlers.lock().unwrap().retain(|(h, _)| h.as_ptr() != handler);
		kResultOk
	}

	pub fn register_timer(&self, handler: *mut ITimerHandler, ms: TimerInterval) -> tresult {
		let Some(handler) = (unsafe { ComRef::from_raw(handler) }) else {
			return kInvalidArgument;
		};
		let interval = Duration::from_millis(ms);
		let next = Instant::now() + interval;
		self.timers
			.lock()
			.unwrap()
			.push(Timer { handler: handler.to_com_ptr(), interval, next });
		kResultOk
	}

	pub fn unregister_timer(&self, handler: *mut ITimerHandler) -> tresult {
		self.timers.lock().unwrap().retain(|t| t.handler.as_ptr() != handler);
		kResultOk
	}

	// Call the handlers of readable file descriptors and due timers. Main thread only.
	// Handlers can (un)register during calls, so nothing is locked while calling.
	pub fn poll(&self) {
		let handlers = self.handlers.lock().unwrap().clone();
		for (handler, fd) in handlers {
			if is_readable(fd) && self.has_handler(&handler) {
				unsafe { handler.onFDIsSet(fd) };
			}
		}

		let now = Instant::now();
		let due: Vec<_> = self
			.timers
			.lock()
			.unwrap()
			.iter_mut()
			.filter(|t| now >= t.next)
			.map(|t| {
				t.next = now + t.interval;
				t.handler.clone()
			})
			.collect();
		for handler in due {
			if self.has_timer(&handler) {
				unsafe { handler.onTimer() };
			}
		}
	}

	fn has_handler(&self, handler: &ComPtr<IEventHandler>) -> bool {
		self.handlers
			.lock()
			.unwrap()
			.iter()
			.any(|(h, _)| h.as_ptr() == handler.as_ptr())
	}

	fn has_timer(&self, handler: &ComPtr<ITimerHandler>) -> bool {
		self.timers
			.lock()
			.unwrap()
			.iter()
			.any(|t| t.handler.as_ptr() == handler.as_ptr())
	}
}

#[cfg(target_os = "linux")]
fn is_readable(fd: FileDescriptor) -> bool {
	let mut pfd = libc::pollfd { fd, events: libc::POLLIN, revents: 0 };
	unsafe { libc::poll(&mut pfd, 1, 0) > 0 && pfd.revents & libc::POLLIN != 0 }
}

#[cfg(not(target_os = "linux"))]
fn is_readable(_fd: FileDescriptor) -> bool {
	false
}
