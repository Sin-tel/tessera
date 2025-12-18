use crate::api::project::Project;
use crate::audio::AUDIO_PANIC;
use crate::context::AudioContext;
use crate::log::*;
use crate::midi;
use crate::opengl::Renderer;
use crate::text::{Font, TextEngine};
use crate::voice_manager::Token;
use femtovg::{Canvas, Color, ImageId, Path};
use semver::Version;
use std::path::PathBuf;
use std::sync::{LazyLock, OnceLock, RwLock, atomic::Ordering, mpsc};
use std::time::Instant;
use winit::window::Window;

pub const INIT_WIDTH: u32 = 1280;
pub const INIT_HEIGHT: u32 = 720;

pub const DEFAULT_FONT_SIZE: f32 = 14.;
pub const DEFAULT_LINE_WIDTH: f32 = 1.5;

pub struct State {
	pub current_color: Color,
	pub line_width: f32,
	pub font: Font,
	pub font_size: f32,
	pub text_engine: TextEngine,
	pub image_ids: Vec<ImageId>,
	pub paths: Vec<Path>,
	pub mouse_position: (f32, f32),
	pub window_size: (u32, u32),
	pub exit: bool,
	pub start_time: Instant,
	pub transform_stack: Vec<femtovg::Transform2D>,
	pub current_scissor: Option<(f32, f32, f32, f32)>,
	pub audio: Option<AudioContext>,
	pub project: Option<Project>,
	pub canvas: Canvas<Renderer>,
	pub window: Window,
	pub scale_factor: f32,
	pub dialog_rx: Option<mpsc::Receiver<Option<PathBuf>>>,
	pub midi_session: Option<midir::MidiInput>,
	pub midi_connections: Vec<midi::Connection>,
	token: Token,
}

impl State {
	pub fn new(canvas: Canvas<Renderer>, window: Window, scale_factor: f32) -> Self {
		State {
			current_color: Color::white(),
			mouse_position: (0., 0.),
			window_size: (INIT_WIDTH, INIT_HEIGHT),
			line_width: DEFAULT_LINE_WIDTH,
			font: Font::Inter,
			font_size: DEFAULT_FONT_SIZE,
			text_engine: TextEngine::new(),
			image_ids: Vec::new(),
			paths: Vec::new(),
			exit: false,
			start_time: std::time::Instant::now(),
			transform_stack: Vec::new(),
			current_scissor: None,
			audio: None,
			project: None,
			token: 0,
			canvas,
			window,
			scale_factor,
			dialog_rx: None,
			midi_session: None,
			midi_connections: Vec::new(),
		}
	}

	pub fn check_audio_status(&mut self) {
		if self.audio.is_some() && AUDIO_PANIC.load(Ordering::Relaxed) {
			log_warn!("Killing backend!");
			AUDIO_PANIC.store(false, Ordering::Relaxed);
			self.audio = None;
		}
	}

	pub fn next_token(&mut self) -> Token {
		self.token = self.token.wrapping_add(1);
		self.token
	}
}

static VERSION: OnceLock<Version> = OnceLock::new();

pub fn get_version() -> &'static Version {
	VERSION.get_or_init(|| {
		Version::parse(env!("CARGO_PKG_VERSION")).expect("Failed to parse version string")
	})
}

pub static NEW_VERSION: LazyLock<RwLock<Option<String>>> = LazyLock::new(|| RwLock::new(None));
const API_URL: &str = "https://api.github.com/repos/sin-tel/tessera/releases/latest";

pub fn check_for_updates() -> Option<String> {
	// Note: Unauthenticated calls are limited to 60 requests/hr per IP.
	// Checking this once at startup is fine.

	let mut resp = match ureq::get(API_URL)
		.header("User-Agent", "tessera-update-checker")
		.call()
	{
		Ok(r) => r,
		Err(e) => {
			log_error!("Request failed: {e}");
			return None;
		},
	};

	let json: serde_json::Value = match resp.body_mut().read_json() {
		Ok(j) => j,
		Err(e) => {
			log_error!("Bad JSON: {e}");
			return None;
		},
	};

	// Extract tag_name (e.g., "v0.1.2")
	let Some(remote_tag) = json["tag_name"].as_str() else {
		log_error!("No tag_name field found");
		return None;
	};

	// Strip the 'v' prefix if present (v0.1.2 -> 0.1.2)
	let clean_remote_version = remote_tag.trim_start_matches('v');

	// Compare versions
	let current = get_version();
	let remote = match Version::parse(clean_remote_version) {
		Ok(v) => v,
		Err(e) => {
			log_error!("Bad remote version format: {e}");
			return None;
		},
	};

	if remote > *current {
		Some(remote_tag.to_string())
	} else {
		log_info!("Version up to date!");
		None
	}
}

pub fn spawn_update_check() {
	// ureq is blocking so spawn a thread

	std::thread::spawn(|| {
		if let Some(tag) = check_for_updates()
			&& let Ok(mut lock) = NEW_VERSION.write()
		{
			*lock = Some(tag);
		}
	});
}
