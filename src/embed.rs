use mlua::prelude::*;
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "assets/"]
pub struct Asset;

#[derive(Embed)]
#[folder = "lua/"]
pub struct Script;

// Add a module loader that works with embedded files
pub fn setup_lua_loader(lua: &Lua) -> LuaResult<()> {
	let loader = lua.create_function(|lua, mod_name: String| {
		// Convert "ui.button" -> "ui/button.lua"
		let path = mod_name.replace('.', "/") + ".lua";

		match Script::get(&path) {
			Some(script) => {
				let loader = lua
					.load(&*script.data)
					.set_name(format!("@lua/{}", path))
					.into_function()?;
				Ok(Some(loader))
			},
			None => Ok(None),
		}
	})?;

	let package: LuaTable = lua.globals().get("package")?;
	let loaders: LuaTable = package.get("loaders")?;

	// Append loader to the end
	let len = loaders.len()?;
	loaders.set(len + 1, loader)?;

	Ok(())
}
