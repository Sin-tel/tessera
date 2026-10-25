use crate::audio::MAX_BUF_SIZE;
use crate::log::*;
use crate::vst3::error::ToResultExt;
use crate::vst3::event::Events;
use crate::vst3::parameter::Parameters;
use crate::vst3::run_loop::RunLoop;
use crate::vst3::scan::PluginDescriptor;
use crate::vst3::scan::guid_from_hex;
use crate::vst3::state::Vst3State;
use crate::vst3::util::extract_cstring_utf16;
use anyhow::{Context, Result, anyhow};
use libloading::{Library, Symbol};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::ffi::c_void;
use std::mem::MaybeUninit;
use std::path::Path;
use std::sync::Arc;
use std::sync::mpsc::SyncSender;
#[cfg(target_os = "linux")]
use vst3::Steinberg::Linux::IRunLoop;
use vst3::Steinberg::Linux::{
	FileDescriptor, IEventHandler, IRunLoopTrait, ITimerHandler, TimerInterval,
};
use vst3::Steinberg::Vst;
use vst3::Steinberg::Vst::ProcessContext_::StatesAndFlags_;
use vst3::Steinberg::kInvalidArgument;
use winit::window::WindowId;

use vst3::Steinberg::Vst::BusInfo_::BusFlags_;
use vst3::Steinberg::Vst::{
	AudioBusBuffers, AudioBusBuffers__type0, BusInfo, Chord, FrameRate, IAudioProcessor,
	IAudioProcessorTrait, IComponent, IComponentHandler, IComponentHandlerTrait, IComponentTrait,
	IConnectionPoint, IConnectionPointTrait, IEditController, IEditControllerTrait,
	IHostApplication, IHostApplicationTrait, IMidiMapping, ProcessContext, ProcessData,
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

pub type EnumType = i32;
const AUDIO: EnumType = Vst::MediaTypes_::kAudio as EnumType;
const OUTPUT: EnumType = Vst::BusDirections_::kOutput as EnumType;
const REALTIME: EnumType = Vst::ProcessModes_::kRealtime as EnumType;
const SAMPLE_32: EnumType = Vst::SymbolicSampleSizes_::kSample32 as EnumType;

struct PluginHost;

impl Class for PluginHost {
	type Interfaces = (IHostApplication,);
}

impl IHostApplicationTrait for PluginHost {
	unsafe fn getName(&self, name: *mut [u16; 128]) -> tresult {
		if name.is_null() {
			return kInvalidArgument;
		}
		let name = unsafe { &mut *name };
		name.fill(0);
		for (dst, src) in name.iter_mut().zip("Tessera".encode_utf16()) {
			*dst = src;
		}
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

// Stub implementation for IComponentHandler
pub struct ComponentHandler;

impl Class for ComponentHandler {
	type Interfaces = (IComponentHandler,);
}

impl IComponentHandlerTrait for ComponentHandler {
	unsafe fn beginEdit(&self, _id: u32) -> tresult {
		kResultOk
	}

	unsafe fn performEdit(&self, _id: u32, _value_normalized: f64) -> tresult {
		kResultOk
	}

	unsafe fn endEdit(&self, _id: u32) -> tresult {
		kResultOk
	}

	unsafe fn restartComponent(&self, _flags: i32) -> tresult {
		kResultOk
	}
}

pub struct PluginFrame {
	window: Arc<Window>,
	run_loop: Arc<RunLoop>,
}

#[cfg(target_os = "linux")]
impl Class for PluginFrame {
	type Interfaces = (IPlugFrame, IRunLoop);
}

#[cfg(not(target_os = "linux"))]
impl Class for PluginFrame {
	type Interfaces = (IPlugFrame,);
}

impl IRunLoopTrait for PluginFrame {
	unsafe fn registerEventHandler(
		&self,
		handler: *mut IEventHandler,
		fd: FileDescriptor,
	) -> tresult {
		self.run_loop.register_event_handler(handler, fd)
	}

	unsafe fn unregisterEventHandler(&self, handler: *mut IEventHandler) -> tresult {
		self.run_loop.unregister_event_handler(handler)
	}

	unsafe fn registerTimer(&self, handler: *mut ITimerHandler, ms: TimerInterval) -> tresult {
		self.run_loop.register_timer(handler, ms)
	}

	unsafe fn unregisterTimer(&self, handler: *mut ITimerHandler) -> tresult {
		self.run_loop.unregister_timer(handler)
	}
}

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

		// no panics in callbacks from the plugin
		let Some(plug_view) = (unsafe { ComRef::from_raw(view) }) else {
			return kInvalidArgument;
		};
		unsafe { plug_view.onSize(new_size) };

		kResultOk
	}
}

type GetPluginFactoryFunc = unsafe extern "system" fn() -> *mut IPluginFactory;

// The plugin's shared library and its factory. Objects created from the factory hold an Arc to this,
// so the module is only exited and unloaded after they're released.
pub struct Vst3Library {
	// released before the module exits
	factory: Option<ComPtr<IPluginFactory>>,
	lib: Library,
}

impl Vst3Library {
	pub fn new(path: &Path) -> Result<Arc<Self>> {
		let lib = unsafe { module_entry(Library::new(path)?)? };

		let factory = unsafe {
			let get_factory: Symbol<GetPluginFactoryFunc> = lib.get(c"GetPluginFactory")?;
			// returned with a reference for us
			ComPtr::from_raw(get_factory())
		};
		let Some(factory) = factory else {
			unsafe { module_exit(&lib) };
			return Err(anyhow!("Plugin returned no factory"));
		};

		Ok(Arc::new(Self { factory: Some(factory), lib }))
	}

	pub fn factory(&self) -> &ComPtr<IPluginFactory> {
		self.factory.as_ref().unwrap()
	}
}

impl Drop for Vst3Library {
	fn drop(&mut self) {
		self.factory = None;
		unsafe { module_exit(&self.lib) };
	}
}

// Module init the SDK requires before using the factory: InitDll (Windows), ModuleEntry (Linux).
// macOS needs bundleEntry with a CFBundleRef, not done yet.
unsafe fn module_entry(lib: Library) -> Result<Library> {
	#[cfg(target_os = "windows")]
	if let Ok(init) = unsafe { lib.get::<unsafe extern "system" fn() -> bool>(c"InitDll") }
		&& !unsafe { init() }
	{
		return Err(anyhow!("InitDll failed"));
	}

	// takes the handle from dlopen
	#[cfg(target_os = "linux")]
	let lib = {
		let handle = libloading::os::unix::Library::from(lib).into_raw();
		let lib = Library::from(unsafe { libloading::os::unix::Library::from_raw(handle) });
		if let Ok(entry) =
			unsafe { lib.get::<unsafe extern "C" fn(*mut c_void) -> bool>(c"ModuleEntry") }
			&& !unsafe { entry(handle) }
		{
			return Err(anyhow!("ModuleEntry failed"));
		}
		lib
	};

	Ok(lib)
}

unsafe fn module_exit(lib: &Library) {
	#[cfg(target_os = "windows")]
	if let Ok(exit) = unsafe { lib.get::<unsafe extern "system" fn() -> bool>(c"ExitDll") } {
		unsafe { exit() };
	}

	#[cfg(target_os = "linux")]
	if let Ok(exit) = unsafe { lib.get::<unsafe extern "C" fn() -> bool>(c"ModuleExit") } {
		unsafe { exit() };
	}

	#[cfg(target_os = "macos")]
	let _ = lib;
}

struct Vst3Window {
	plug_view: ComPtr<IPlugView>,
	// setFrame doesn't add a reference, so we keep the frame alive
	_frame: ComPtr<IPlugFrame>,
	run_loop: Arc<RunLoop>,
	id: WindowId,
}

// Plugin objects shared by the editor and the processor. Torn down when both are gone, in the order
// the SDK wants: disconnect, terminate controller, terminate component.
struct PluginInstance {
	component: ComPtr<IComponent>,
	// only when it's a separate object from the component
	controller: Option<ComPtr<IEditController>>,
	connection: Option<(ComPtr<IConnectionPoint>, ComPtr<IConnectionPoint>)>,
	_host: ComWrapper<PluginHost>,
	_handler: ComWrapper<ComponentHandler>,
	// last, so the module is unloaded after everything else is released
	_lib: Arc<Vst3Library>,
}

impl Drop for PluginInstance {
	fn drop(&mut self) {
		unsafe {
			if let Some((processor, controller)) = self.connection.take() {
				let _ = processor.disconnect(controller.as_ptr());
				let _ = controller.disconnect(processor.as_ptr());
			}
			if let Some(controller) = self.controller.take() {
				let _ = controller.terminate();
			}
			let _ = self.component.terminate();
		}
	}
}

pub struct Vst3Editor {
	id: usize,
	name: String,
	window: Option<Vst3Window>,
	edit_controller: ComPtr<IEditController>,
	_instance: Arc<PluginInstance>,
}

pub struct Vst3Processor {
	id: usize,
	sample_rate: f32,
	cleanup_tx: SyncSender<usize>,
	process_failed: bool,
	pub events: Events,
	pub parameters: Parameters,
	audio_processor: ComPtr<IAudioProcessor>,
	instance: Arc<PluginInstance>,
}

// Create an object of class `cid` from the factory.
fn create_instance<I: Interface>(lib: &Vst3Library, cid: &[i8; 16]) -> Result<ComPtr<I>> {
	let mut obj: *mut c_void = std::ptr::null_mut();
	unsafe {
		lib.factory()
			.createInstance(cid.as_ptr(), I::IID.as_ptr() as *const i8, &mut obj)
	}
	.as_result()?;
	// returned with a reference for us
	unsafe { ComPtr::from_raw(obj as *mut I) }
		.ok_or_else(|| anyhow!("createInstance returned null"))
}

pub fn load(
	plugin: &PluginDescriptor,
	sample_rate: f32,
	id: usize,
	cleanup_tx: SyncSender<usize>,
) -> Result<(Vst3Editor, Vst3Processor)> {
	let lib = Vst3Library::new(&plugin.library_path)?;
	let plugin_cid = guid_from_hex(&plugin.guid)?;

	let component: ComPtr<IComponent> =
		create_instance(&lib, &plugin_cid).context("Failed to create component")?;

	let host = ComWrapper::new(PluginHost);
	let host_ptr = host.to_com_ptr::<IHostApplication>().unwrap();
	let host_unknown = host_ptr.as_ptr() as *mut vst3::Steinberg::FUnknown;

	unsafe { component.initialize(host_unknown) }
		.as_result()
		.context("Failed to initialize component")?;

	let handler = ComWrapper::new(ComponentHandler);
	let handler_ptr = handler.to_com_ptr::<IComponentHandler>().unwrap();

	// from here on, dropping the instance terminates what was initialized
	let mut instance = PluginInstance {
		component,
		controller: None,
		connection: None,
		_host: host,
		_handler: handler,
		_lib: Arc::clone(&lib),
	};

	let audio_processor = instance
		.component
		.cast::<IAudioProcessor>()
		.ok_or_else(|| anyhow!("Component does not implement IAudioProcessor"))?;

	let edit_controller = if let Some(controller) = instance.component.cast::<IEditController>() {
		// single component: it's also the controller, already initialized
		controller
	} else {
		let mut controller_cid = [0i8; 16];
		unsafe { instance.component.getControllerClassId(&mut controller_cid) }
			.as_result()
			.context("Failed to get controller class ID")?;

		let controller: ComPtr<IEditController> = create_instance(&lib, &controller_cid)
			.context("Failed to create IEditController instance")?;
		unsafe { controller.initialize(host_unknown) }
			.as_result()
			.context("Failed to initialize controller")?;
		instance.controller = Some(controller.clone());

		// connect processor and controller
		let (Some(c1), Some(c2)) =
			(instance.component.cast::<IConnectionPoint>(), controller.cast::<IConnectionPoint>())
		else {
			return Err(anyhow!("Plugin does not support IConnectionPoint"));
		};
		unsafe { c1.connect(c2.as_ptr()) }
			.as_result()
			.context("Failed to connect")?;
		// stored now so a failure below still disconnects c1
		instance.connection = Some((c1.clone(), c2.clone()));
		unsafe { c2.connect(c1.as_ptr()) }
			.as_result()
			.context("Failed to connect")?;

		controller
	};

	unsafe { edit_controller.setComponentHandler(handler_ptr.as_ptr()) }
		.as_result()
		.context("Failed to set Component Handler")?;

	let midi_mapping = edit_controller
		.cast::<IMidiMapping>()
		.ok_or_else(|| anyhow!("Plugin doesn't support IMidiMapping."))?;
	let parameters = Parameters::new(midi_mapping.as_com_ref())?;

	let component = &instance.component;

	// Tell it about audio engine settings
	let mut setup = ProcessSetup {
		processMode: REALTIME,
		symbolicSampleSize: SAMPLE_32,
		maxSamplesPerBlock: MAX_BUF_SIZE as i32,
		sampleRate: f64::from(sample_rate),
	};

	unsafe { audio_processor.setupProcessing(&mut setup) }
		.as_result()
		.context("Failed to setup processing")?;

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
		log_warn!("Default stereo bus arrangement not accepted.");
		let bus_count = unsafe { component.getBusCount(AUDIO, OUTPUT) };

		for i in 0..bus_count {
			let mut bus_info = MaybeUninit::<BusInfo>::uninit();
			unsafe { component.getBusInfo(AUDIO, OUTPUT, i, bus_info.as_mut_ptr()) }.as_result()?;
			let bus_info = unsafe { bus_info.assume_init() };

			log_info!(
				"bus: {i} name: {:?} channelCount: {:?} default: {:?}",
				extract_cstring_utf16(&bus_info.name),
				bus_info.channelCount,
				bus_info.flags & BusFlags_::kDefaultActive as u32 > 0,
			);
		}
	}

	// Activate bus 0
	unsafe { component.activateBus(AUDIO, OUTPUT, 0, 1) }
		.as_result()
		.context("Failed to activate audio output bus")?;
	unsafe { component.setActive(1) }
		.as_result()
		.context("Failed to set component active")?;
	if let Err(e) = unsafe { audio_processor.setProcessing(1) }.as_result() {
		unsafe { component.setActive(0) };
		return Err(e).context("Failed to set audio processing state");
	}

	let instance = Arc::new(instance);

	let editor = Vst3Editor {
		id,
		name: plugin.name.clone(),
		window: None,
		edit_controller,
		_instance: Arc::clone(&instance),
	};

	let processor = Vst3Processor {
		id,
		sample_rate,
		cleanup_tx,
		process_failed: false,
		events: Events::new(),
		parameters,
		audio_processor,
		instance,
	};

	Ok((editor, processor))
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

	pub fn set_state(&self, state: &Vst3State) -> Result<()> {
		state.rewind();
		#[allow(non_upper_case_globals)]
		match unsafe { self.edit_controller.setComponentState(state.as_ptr()) } {
			kResultOk | kNotImplemented => (),
			other => other.as_result()?,
		};
		Ok(())
	}

	pub fn open_window(&mut self, window: Arc<Window>) -> Result<()> {
		let view_ptr = unsafe { self.edit_controller.createView(ViewType::kEditor) };
		// returned with a reference for us
		let plug_view = unsafe { ComPtr::<IPlugView>::from_raw(view_ptr) }
			.ok_or_else(|| anyhow!("Plugin does not have a GUI!"))?;

		let raw_window_handle = window.window_handle()?.as_raw();

		// Get platform specific handle
		let (system_window_handle, platform_type) = match raw_window_handle {
			#[cfg(target_os = "windows")]
			RawWindowHandle::Win32(handle) => (handle.hwnd.get() as *mut c_void, kPlatformTypeHWND),
			#[cfg(target_os = "macos")]
			RawWindowHandle::AppKit(handle) => (handle.ns_view.as_ptr() as *mut c_void, kPlatformTypeNSView),
			#[cfg(target_os = "linux")]
			RawWindowHandle::Xlib(handle) => (handle.window as *mut c_void, kPlatformTypeX11EmbedWindowID),
			_ => return Err(anyhow!("Unsupported platform.")),
		};

		unsafe { plug_view.isPlatformTypeSupported(platform_type) }
			.as_result()
			.context("Plugin GUI doesn't support this platform")?;

		// The frame goes first: plugins look up the run loop through it in attached().
		let run_loop = Arc::new(RunLoop::default());
		let frame = ComWrapper::new(PluginFrame {
			window: Arc::clone(&window),
			run_loop: Arc::clone(&run_loop),
		})
		.to_com_ptr::<IPlugFrame>()
		.unwrap();
		unsafe { plug_view.setFrame(frame.as_ptr()) }
			.as_result()
			.context("Failed to set frame")?;

		if let Err(e) =
			unsafe { plug_view.attached(system_window_handle, platform_type) }.as_result()
		{
			unsafe { plug_view.setFrame(std::ptr::null_mut()) };
			return Err(e).context("Failed to attach view");
		}

		// Set window to initial size
		let mut view_rect = vst3::Steinberg::ViewRect { left: 0, top: 0, right: 0, bottom: 0 };
		unsafe {
			if plug_view.getSize(&mut view_rect) == kResultOk {
				let width = view_rect.right - view_rect.left;
				let height = view_rect.bottom - view_rect.top;
				let _ = window.request_inner_size(winit::dpi::PhysicalSize::new(width, height));
			}
		}

		self.window = Some(Vst3Window { plug_view, _frame: frame, run_loop, id: window.id() });

		window.set_visible(true);

		Ok(())
	}

	// Linux only: run the GUI's event handlers and timers. Call from the main thread.
	pub fn poll_run_loop(&self) {
		if let Some(window) = &self.window {
			window.run_loop.poll();
		}
	}

	pub fn close_window(&mut self) {
		if let Some(window) = self.window.take() {
			unsafe {
				let _ = window.plug_view.removed();
				let _ = window.plug_view.setFrame(std::ptr::null_mut());
			}
		}
	}
}

impl Drop for Vst3Editor {
	fn drop(&mut self) {
		// the view goes before the instance terminates the controller
		self.close_window();
	}
}

impl Vst3Processor {
	pub fn id(&self) -> usize {
		self.id
	}

	pub fn get_state(&self) -> Result<Vst3State> {
		let state = Vst3State::new();
		unsafe { self.instance.component.getState(state.as_ptr()) }.as_result()?;
		Ok(state)
	}

	pub fn set_state(&self, state: &Vst3State) -> Result<()> {
		state.rewind();
		unsafe { self.instance.component.setState(state.as_ptr()) }.as_result()?;
		Ok(())
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

		let mut context = ProcessContext {
			// TODO: Send correct playing flag
			state: (StatesAndFlags_::kPlaying as u32)
				| (StatesAndFlags_::kTempoValid as u32)
				| (StatesAndFlags_::kProjectTimeMusicValid as u32),

			sampleRate: f64::from(self.sample_rate),

			// TODO: hardcoded for now
			tempo: 120.0,

			// TODO (Transport quarter notes)
			projectTimeMusic: 0.0,
			projectTimeSamples: 0,

			// Other fields we don't care about
			barPositionMusic: 0.0,
			cycleStartMusic: 0.0,
			cycleEndMusic: 0.0,
			timeSigNumerator: 4,
			timeSigDenominator: 4,
			systemTime: 0,
			smpteOffsetSubframes: 0,
			frameRate: FrameRate { framesPerSecond: 60, flags: 0 },
			chord: Chord { keyNote: 0, rootNote: 0, chordMask: 0 },
			samplesToNextClock: 0,
			continousTimeSamples: 0,
		};

		// Populate buffer process data
		let mut process_data = ProcessData {
			processMode: REALTIME,
			symbolicSampleSize: SAMPLE_32,
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
			inputParameterChanges: self.parameters.as_com_ptr(),
			outputParameterChanges: std::ptr::null_mut(),

			// Optional according to docs, but might be required for some plugins to work properly
			processContext: &mut context,
		};

		// Run processing
		let res = unsafe { self.audio_processor.process(&mut process_data) };

		// only once, this runs every block
		if let Err(e) = res.as_result()
			&& !self.process_failed
		{
			self.process_failed = true;
			log_error!("VST processing failed: {e}");
		}

		// Clear the queue for the next call
		self.events.clear();
		self.parameters.clear();
	}

	pub fn flush(&self) -> Result<()> {
		unsafe { self.audio_processor.setProcessing(0) }.as_result()?;
		unsafe { self.audio_processor.setProcessing(1) }.as_result()?;
		Ok(())
	}
}

impl Drop for Vst3Processor {
	fn drop(&mut self) {
		// terminated by the instance, once the editor is gone too
		unsafe {
			let _ = self.audio_processor.setProcessing(0);
			let _ = self.instance.component.setActive(0);
		}

		// Note: this may block, but dropping the processor is not realtime safe anyway.
		let _ = self.cleanup_tx.send(self.id);
	}
}
