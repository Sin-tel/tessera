use crate::audio::MAX_BUF_SIZE;
use crate::vst3::error::ToResultExt;
use crate::vst3::event::Events;
use crate::vst3::parameter::Parameters;
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
use vst3::Steinberg::Vst;
use vst3::Steinberg::Vst::ProcessContext_::StatesAndFlags_;
use vst3::Steinberg::kInvalidArgument;
use winit::window::WindowId;

use vst3::Steinberg::Vst::BusInfo_::BusFlags_;
use vst3::Steinberg::Vst::{
	AudioBusBuffers, AudioBusBuffers__type0, BusInfo, Chord, FrameRate, IAudioProcessor,
	IAudioProcessorTrait, IComponent, IComponentTrait, IConnectionPoint, IConnectionPointTrait,
	IEditController, IEditControllerTrait, IHostApplication, IHostApplicationTrait, IMidiMapping,
	ProcessContext, ProcessData, ProcessSetup, SpeakerArr, ViewType,
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
	pub fn new(path: &Path) -> Result<Arc<Self>> {
		let lib = unsafe { Library::new(path)? };

		unsafe {
			if let Ok(init_dll) = lib.get::<unsafe extern "system" fn() -> bool>(c"InitDll") {
				init_dll();
			}
		}

		Ok(Arc::new(Self { lib }))
	}

	pub fn get_factory(&self) -> Result<*mut vst3::Steinberg::FUnknown> {
		let get_factory: Symbol<GetPluginFactoryFunc> =
			unsafe { self.lib.get(c"GetPluginFactory")? };
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
	sample_rate: f32,
	cleanup_tx: SyncSender<usize>,
	pub events: Events,
	pub parameters: Parameters,
	audio_processor: ComPtr<IAudioProcessor>,
	component: ComPtr<IComponent>,
	lib: Arc<Vst3Library>,
}

pub fn load(
	plugin: &PluginDescriptor,
	sample_rate: f32,
	id: usize,
	cleanup_tx: SyncSender<usize>,
) -> Result<(Vst3Editor, Vst3Processor)> {
	let lib = Vst3Library::new(&plugin.library_path)?;

	// Get the factory
	let factory_ptr = lib.get_factory()?;
	let factory = unsafe { ComRef::<IPluginFactory>::from_raw(factory_ptr as *mut _).unwrap() };

	let plugin_cid = guid_from_hex(&plugin.guid)?;

	// Create the processor IComponent
	let mut component_ptr: *mut c_void = std::ptr::null_mut();
	unsafe {
		factory.createInstance(
			plugin_cid.as_ptr(),
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
		.as_result()
		.context("Failed to initialize component")?;

	// Query the IAudioProcessor interface
	let audio_processor = component
		.cast::<IAudioProcessor>()
		.expect("Component does not implement IAudioProcessor");

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
		println!("Default stereo bus arrangement not accepted.");
		let bus_count = unsafe { component.getBusCount(AUDIO, OUTPUT) };

		println!("Output bus count: {bus_count:?}");

		for i in 0..bus_count {
			let mut bus_info = MaybeUninit::<BusInfo>::uninit();
			unsafe { component.getBusInfo(AUDIO, OUTPUT, i, bus_info.as_mut_ptr()) }.as_result()?;
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
	unsafe { component.activateBus(AUDIO, OUTPUT, 0, 1) }
		.as_result()
		.context("Failed to activate audio output bus")?;
	unsafe { component.setActive(1) }
		.as_result()
		.context("Failed to set component active")?;
	unsafe { audio_processor.setProcessing(1) }
		.as_result()
		.context("Failed to set audio processing state")?;

	// Setup editor
	let edit_controller = if let Some(editor) = component.cast::<IEditController>() {
		// Single component: processor implements IEditController directly
		editor
	} else {
		// Multiple components, query for controller class
		let mut editor_cid = [0i8; 16];
		unsafe { component.getControllerClassId(&mut editor_cid) }
			.as_result()
			.context("Failed to get controller class ID")?;

		// Create the editor instance
		let mut editor_ptr: *mut c_void = std::ptr::null_mut();
		unsafe {
			factory.createInstance(
				editor_cid.as_ptr(),
				IEditController::IID.as_ptr() as *const i8,
				&mut editor_ptr,
			)
		}
		.as_result()
		.context("Failed to create IEditController instance")?;

		let edit_controller =
			unsafe { ComPtr::from_raw(editor_ptr as *mut IEditController).unwrap() };

		// Initialize editor in host context
		unsafe { edit_controller.initialize(host_ptr.as_ptr() as *mut vst3::Steinberg::FUnknown) }
			.as_result()
			.context("Failed to initialize controller")?;

		// Wire them up using IConnectionPoint
		let audio_connection = audio_processor.cast::<IConnectionPoint>();
		let edit_connection = edit_controller.cast::<IConnectionPoint>();

		if let (Some(c1), Some(c2)) = (audio_connection, edit_connection) {
			unsafe {
				c1.connect(c2.as_ptr()).as_result().unwrap();
				c2.connect(c1.as_ptr()).as_result().unwrap();
			}
		} else {
			return Err(anyhow!("Plugin does not support IConnectionPoint"));
		}

		edit_controller
	};

	let midi_mapping = edit_controller
		.cast::<IMidiMapping>()
		.ok_or_else(|| anyhow!("Plugin doesn't support IMidiMapping."))?;

	let parameters = Parameters::new(midi_mapping.as_com_ref())?;

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
		sample_rate,
		cleanup_tx,
		events: Events::new(),
		parameters,
		audio_processor,
		component,
		lib: Arc::clone(&lib),
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
		state.rewind()?;
		#[allow(non_upper_case_globals)]
		match unsafe { self.edit_controller.setComponentState(state.as_ptr()) } {
			kResultOk | kNotImplemented => Ok(()),
			other => Ok(other.as_result()?),
		}
	}

	pub fn open_window(&mut self, window: Arc<Window>) -> Result<()> {
		let view_ptr = unsafe { self.edit_controller.createView(ViewType::kEditor) };
		if view_ptr.is_null() {
			return Err(anyhow!("Plugin does not have a GUI!"));
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
			_ => return Err(anyhow!("Unsupported platform.")),
		};

		// Attach handle
		let plug_view = unsafe { ComPtr::from_raw(view_ptr).unwrap() };
		unsafe { plug_view.attached(system_window_handle, platform_type) }
			.as_result()
			.context("Failed to attach view")?;

		// Setup frame
		let frame = ComWrapper::new(PluginFrame { window: Arc::clone(&window) })
			.to_com_ptr::<IPlugFrame>()
			.unwrap();
		unsafe { plug_view.setFrame(frame.as_ptr()) }
			.as_result()
			.context("Failed to set frame")?;

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

impl Drop for Vst3Editor {
	fn drop(&mut self) {
		unsafe {
			let _ = self.edit_controller.terminate();
		}
	}
}

impl Vst3Processor {
	pub fn id(&self) -> usize {
		self.id
	}

	pub fn get_state(&self) -> Result<Vst3State> {
		let state = Vst3State::new();
		unsafe { self.component.getState(state.as_ptr()) }.as_result()?;
		Ok(state)
	}

	pub fn set_state(&self, state: &Vst3State) -> Result<()> {
		state.rewind()?;
		Ok(unsafe { self.component.setState(state.as_ptr()) }.as_result()?)
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

		// Log error without panic
		if let Err(e) = res.as_result() {
			eprintln!("Audio processing failed: {e}");
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
		unsafe {
			let _ = self.audio_processor.setProcessing(0);
			let _ = self.component.setActive(0);
			let _ = self.component.terminate();
		}

		// Note: this may block, but dropping the processor is not realtime safe anyway.
		let _ = self.cleanup_tx.send(self.id);
	}
}
