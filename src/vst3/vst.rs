use crate::audio::MAX_BUF_SIZE;
use crate::vst3::error::ToResultExt;
use crate::vst3::event::EventQueue;
use crate::vst3::parameter::AutomationQueue;
use crate::vst3::scan::PluginDescriptor;
use crate::vst3::util::extract_cstring_utf16;
use libloading::{Library, Symbol};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::ffi::c_void;
use std::mem::MaybeUninit;
use std::path::Path;
use std::sync::Arc;
use std::sync::mpsc::SyncSender;
use vst3::Steinberg::Vst::MediaTypes_::kAudio;
use vst3::Steinberg::kInvalidArgument;
use winit::window::WindowId;

use vst3::Steinberg::Vst::BusDirections_::kOutput;
use vst3::Steinberg::Vst::BusInfo_::BusFlags_;
use vst3::Steinberg::Vst::ProcessModes_::kRealtime;
use vst3::Steinberg::Vst::SymbolicSampleSizes_::kSample32;
use vst3::Steinberg::Vst::{
	AudioBusBuffers, AudioBusBuffers__type0, BusInfo, IAudioProcessor, IAudioProcessorTrait,
	IComponent, IComponentTrait, IConnectionPoint, IConnectionPointTrait, IEditController,
	IEditControllerTrait, IHostApplication, IHostApplicationTrait, IMidiMapping, ProcessData,
	ProcessSetup, SpeakerArr, ViewType,
};
use vst3::Steinberg::{
	IPlugFrame, IPlugFrameTrait, IPlugView, IPlugViewTrait, IPluginBaseTrait, IPluginFactory,
	IPluginFactoryTrait, ViewRect,
};
use vst3::Steinberg::{kNotImplemented, kResultOk, tresult};
#[allow(unused_imports)]
use vst3::Steinberg::{kPlatformTypeHWND, kPlatformTypeNSView, kPlatformTypeX11EmbedWindowID};
use vst3::com_scrape_types::{Class, ComRef, ComWrapper};
use vst3::{ComPtr, Interface};
use winit::window::Window;

struct PluginHost;

impl Class for PluginHost {
	type Interfaces = (IHostApplication,);
}

impl IHostApplicationTrait for PluginHost {
	unsafe fn getName(&self, _name: *mut [u16; 128]) -> tresult {
		// TODO: write something to name
		kResultOk
	}

	unsafe fn createInstance(
		&self,
		_cid: *mut [i8; 16],
		_iid: *mut [i8; 16],
		_obj: *mut *mut std::ffi::c_void,
	) -> tresult {
		// TODO
		kNotImplemented
	}
}

pub struct PluginFrame {
	window: Arc<Window>,
}

impl Class for PluginFrame {
	type Interfaces = (IPlugFrame,);
}

// TODO: implement IRunLoopTrait on linux
impl IPlugFrameTrait for PluginFrame {
	unsafe fn resizeView(
		&self,
		view: *mut vst3::Steinberg::IPlugView,
		new_size: *mut ViewRect,
	) -> tresult {
		if new_size.is_null() {
			return kInvalidArgument;
		}

		let rect = unsafe { *new_size };
		let width = rect.right - rect.left;
		let height = rect.bottom - rect.top;
		let _ = self
			.window
			.request_inner_size(winit::dpi::PhysicalSize::new(width, height));

		let plug_view = unsafe { ComRef::from_raw(view).unwrap() };
		unsafe { plug_view.onSize(new_size) };

		kResultOk
	}
}

type GetPluginFactoryFunc = unsafe extern "system" fn() -> *mut vst3::Steinberg::FUnknown;

pub struct Vst3Library {
	lib: Library,
}

impl Vst3Library {
	pub fn new(path: &Path) -> Result<Arc<Self>, String> {
		let lib = unsafe { Library::new(path).map_err(|e| e.to_string())? };

		unsafe {
			if let Ok(init_dll) = lib.get::<unsafe extern "system" fn() -> bool>(c"InitDll") {
				init_dll();
			}
		}

		Ok(Arc::new(Self { lib }))
	}

	pub fn get_factory(&self) -> Result<*mut vst3::Steinberg::FUnknown, String> {
		let get_factory: Symbol<GetPluginFactoryFunc> =
			unsafe { self.lib.get(c"GetPluginFactory").map_err(|e| e.to_string())? };
		let factory_ptr = unsafe { get_factory() };
		assert!(!factory_ptr.is_null());
		Ok(factory_ptr)
	}
}

impl Drop for Vst3Library {
	fn drop(&mut self) {
		unsafe {
			if let Ok(exit_dll) = self.lib.get::<unsafe extern "system" fn() -> bool>(c"ExitDll") {
				exit_dll();
			}
		}
	}
}

#[allow(unused)]
struct Vst3Window {
	plug_view: ComPtr<IPlugView>,
	frame: ComPtr<IPlugFrame>,
	id: WindowId,
}

#[allow(unused)]
pub struct Vst3Editor {
	id: usize,
	name: String,
	window: Option<Vst3Window>,
	edit_controller: ComPtr<IEditController>,
	host_context: ComWrapper<PluginHost>,
	lib: Arc<Vst3Library>,
}

#[allow(unused)]
pub struct Vst3Processor {
	id: usize,
	cleanup_tx: SyncSender<usize>,
	pub events: EventQueue,
	pub automation: AutomationQueue,
	audio_processor: ComPtr<IAudioProcessor>,
	component: ComPtr<IComponent>,
	lib: Arc<Vst3Library>,
}

pub fn load(
	plugin: &PluginDescriptor,
	sample_rate: f32,
	id: usize,
	cleanup_tx: SyncSender<usize>,
) -> Result<(Vst3Editor, Vst3Processor), String> {
	let lib = Vst3Library::new(&plugin.library_path)?;

	// Get the factory
	let factory_ptr = lib.get_factory()?;
	let factory = unsafe { ComRef::<IPluginFactory>::from_raw(factory_ptr as *mut _).unwrap() };

	// Create the processor IComponent
	let mut component_ptr: *mut c_void = std::ptr::null_mut();
	unsafe {
		factory.createInstance(
			plugin.processor_cid.as_ptr(),
			IComponent::IID.as_ptr() as *const i8,
			&mut component_ptr,
		);
	}
	let component = unsafe { ComPtr::from_raw(component_ptr as *mut IComponent).unwrap() };

	// Create host context
	let host_context = ComWrapper::new(PluginHost);
	let host_ptr = host_context.to_com_ptr::<IHostApplication>().unwrap();

	// Initialize the plugin in host context
	unsafe { component.initialize(host_ptr.as_ptr() as *mut vst3::Steinberg::FUnknown) }
		.as_result()?;

	// Query the IAudioProcessor interface
	let audio_processor = component
		.cast::<IAudioProcessor>()
		.expect("Component does not implement IAudioProcessor");

	// Tell it about audio engine settings
	let mut setup = ProcessSetup {
		processMode: kRealtime,
		symbolicSampleSize: kSample32,
		maxSamplesPerBlock: MAX_BUF_SIZE as i32,
		sampleRate: f64::from(sample_rate),
	};

	unsafe { audio_processor.setupProcessing(&mut setup) }.as_result()?;

	let res = unsafe {
		audio_processor.setBusArrangements(
			// input
			std::ptr::null_mut(),
			0,
			// output
			&SpeakerArr::kStereo as *const _ as *mut _,
			1,
		)
	};
	if res != kResultOk {
		println!("Default stereo bus arrangement not accepted.");
		let bus_count = unsafe { component.getBusCount(kAudio, kOutput) };

		println!("Output bus count: {bus_count:?}");

		for i in 0..bus_count {
			let mut bus_info = MaybeUninit::<BusInfo>::uninit();
			unsafe { component.getBusInfo(kAudio, kOutput, i, bus_info.as_mut_ptr()) }
				.as_result()?;
			let bus_info = unsafe { bus_info.assume_init() };

			println!(
				"bus: {i} name: {:?} channelCount: {:?} default: {:?}",
				extract_cstring_utf16(&bus_info.name),
				bus_info.channelCount,
				bus_info.flags & BusFlags_::kDefaultActive as u32 > 0,
			);
		}
	}

	// Activate bus 0
	unsafe { component.activateBus(kAudio, kOutput, 0, 1) }.as_result()?;
	unsafe { component.setActive(1) }.as_result()?;
	unsafe { audio_processor.setProcessing(1) }.as_result()?;

	// Setup editor
	let edit_controller = if let Some(editor) = component.cast::<IEditController>() {
		// Single component: processor implements IEditController directly
		editor
	} else {
		// Multiple components, query for controller class
		let mut editor_cid = [0i8; 16];
		unsafe { component.getControllerClassId(&mut editor_cid) }.as_result()?;

		// Create the editor instance
		let mut editor_ptr: *mut c_void = std::ptr::null_mut();
		unsafe {
			factory.createInstance(
				editor_cid.as_ptr(),
				IEditController::IID.as_ptr() as *const i8,
				&mut editor_ptr,
			)
		}
		.as_result()?;

		let edit_controller =
			unsafe { ComPtr::from_raw(editor_ptr as *mut IEditController).unwrap() };

		// Initialize editor in host context
		unsafe { edit_controller.initialize(host_ptr.as_ptr() as *mut vst3::Steinberg::FUnknown) }
			.as_result()?;

		// Wire them up using IConnectionPoint
		let audio_connection = audio_processor.cast::<IConnectionPoint>();
		let edit_connection = edit_controller.cast::<IConnectionPoint>();

		if let (Some(c1), Some(c2)) = (audio_connection, edit_connection) {
			unsafe {
				c1.connect(c2.as_ptr()).as_result()?;
				c2.connect(c1.as_ptr()).as_result()?;
			}
		} else {
			return Err("Plugin does not support IConnectionPoint".into());
		}

		edit_controller
	};

	let midi_mapping = edit_controller
		.cast::<IMidiMapping>()
		.ok_or("Plugin doesn't support IMidiMapping.")?;

	let automation = AutomationQueue::new(midi_mapping.as_com_ref())?;

	let editor = Vst3Editor {
		id,
		name: plugin.name.clone(),
		window: None,
		edit_controller,
		host_context,
		lib: Arc::clone(&lib),
	};

	let processor = Vst3Processor {
		id,
		cleanup_tx,
		events: EventQueue::new(),
		automation,
		audio_processor,
		component,
		lib: Arc::clone(&lib),
	};

	Ok((editor, processor))
}

impl Drop for Vst3Processor {
	fn drop(&mut self) {
		// Note: this may block, but dropping the processor is not realtime safe anyway.
		let _ = self.cleanup_tx.send(self.id);
	}
}

impl Vst3Editor {
	pub fn id(&self) -> usize {
		self.id
	}

	pub fn name(&self) -> String {
		self.name.clone()
	}

	pub fn window_id(&self) -> Option<WindowId> {
		self.window.as_ref().map(|w| w.id)
	}

	pub fn open_window(&mut self, window: Arc<Window>) -> Result<(), String> {
		let view_ptr = unsafe { self.edit_controller.createView(ViewType::kEditor) };
		if view_ptr.is_null() {
			return Err("Plugin does not have a GUI!".into());
		}

		let raw_window_handle = window.window_handle().ok().map(|wh| wh.as_raw()).unwrap();

		// Get platform specific handle
		let (system_window_handle, platform_type) = match raw_window_handle {
			#[cfg(target_os = "windows")]
			RawWindowHandle::Win32(handle) => (handle.hwnd.get() as *mut c_void, kPlatformTypeHWND),
			#[cfg(target_os = "macos")]
			RawWindowHandle::AppKit(handle) => (handle.ns_view.as_ptr() as *mut c_void, kPlatformTypeNSView),
			#[cfg(target_os = "linux")]
			RawWindowHandle::Xlib(handle) => (handle.window as *mut c_void, kPlatformTypeX11EmbedWindowID),
			_ => return Err("Unsupported platform.".into()),
		};

		// Attach handle
		let plug_view = unsafe { ComPtr::from_raw(view_ptr).unwrap() };
		unsafe { plug_view.attached(system_window_handle, platform_type) }.as_result()?;

		// Setup frame
		let frame = ComWrapper::new(PluginFrame { window: Arc::clone(&window) })
			.to_com_ptr::<IPlugFrame>()
			.unwrap();
		unsafe { plug_view.setFrame(frame.as_ptr()) }.as_result()?;

		// Set window to initial size
		let mut view_rect = vst3::Steinberg::ViewRect { left: 0, top: 0, right: 0, bottom: 0 };
		unsafe {
			if plug_view.getSize(&mut view_rect) == kResultOk {
				let width = view_rect.right - view_rect.left;
				let height = view_rect.bottom - view_rect.top;
				let _ = window.request_inner_size(winit::dpi::PhysicalSize::new(width, height));
			}
		}

		self.window = Some(Vst3Window { plug_view, frame, id: window.id() });

		window.set_visible(true);
		// window.request_redraw();

		Ok(())
	}

	pub fn close_window(&mut self) {
		self.window = None;
	}
}

impl Vst3Processor {
	pub fn id(&self) -> usize {
		self.id
	}

	pub fn process(&mut self, left_buf: &mut [f32], right_buf: &mut [f32]) {
		// VST3 wants a pointer to an array of channel pointers
		let mut channels = [left_buf.as_mut_ptr(), right_buf.as_mut_ptr()];

		let buf_size = left_buf.len();
		assert!(left_buf.len() == right_buf.len());
		assert!(buf_size <= MAX_BUF_SIZE);

		let mut output_bus = AudioBusBuffers {
			numChannels: 2,
			silenceFlags: 0,
			__field0: AudioBusBuffers__type0 { channelBuffers32: channels.as_mut_ptr() },
		};

		// Populate buffer process data
		let mut process_data = ProcessData {
			processMode: kRealtime,
			symbolicSampleSize: kSample32,
			numSamples: buf_size as i32,

			// Audio input
			numInputs: 0,
			inputs: std::ptr::null_mut(),

			// Audio output
			numOutputs: 1, // 1 stereo bus
			outputs: &mut output_bus,

			// Events
			inputEvents: self.events.as_com_ptr(),
			outputEvents: std::ptr::null_mut(),

			// Parameters
			inputParameterChanges: self.automation.as_com_ptr(),
			outputParameterChanges: std::ptr::null_mut(),

			// Optional according to docs, but might be required for some plugins to work properly
			processContext: std::ptr::null_mut(),
		};

		// Run processing
		let res = unsafe { self.audio_processor.process(&mut process_data) };

		// Log error without panic
		if let Err(e) = res.as_result() {
			eprintln!("Audio processing failed: {e}");
		}

		// Clear the queue for the next call
		self.events.clear();
		self.automation.clear();
	}
}
