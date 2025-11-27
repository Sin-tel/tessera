const INIT_WIDTH: u32 = 800;
const INIT_HEIGHT: u32 = 600;

use mlua::Lua;
use mlua::prelude::*;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
	pub src_dir: PathBuf,
	pub width: u32,
	pub height: u32,
	pub title: String,
	pub resizeable: bool,
}

impl Config {
	pub fn read(src_dir: PathBuf) -> LuaResult<Self> {
		let mut config = Self {
			src_dir,
			width: INIT_WIDTH,
			height: INIT_HEIGHT,
			title: "Untitled".to_string(),
			resizeable: false,
		};

		if let Ok(config_file) = fs::read_to_string(config.src_dir.join("conf.lua")) {
			let lua = Lua::new();
			let love = lua.create_table()?;
			lua.globals().set("love", &love)?;

			lua.load(config_file).exec()?;

			let config_fn: LuaFunction = love.get("conf")?;
			let config_table = lua.create_table()?;
			let config_window = lua.create_table()?;
			config_table.set("window", &config_window)?;
			// not used
			config_table.set("modules", lua.create_table()?)?;

			config_fn.call::<()>(&config_table)?;

			config.title = config_window.get("title").unwrap_or("Untitled".to_string());
			config.width = config_window.get("width").unwrap_or(INIT_WIDTH);
			config.height = config_window.get("height").unwrap_or(INIT_HEIGHT);
			config.resizeable = config_window.get("resizable").unwrap_or(false);
		}

		Ok(config)
	}
}
