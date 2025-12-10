use crate::app::State;
use crate::embed::Asset;
use femtovg::ImageFlags;
use mlua::prelude::*;

#[rustfmt::skip]
const BUILTIN_IMG: &[(&str, &str)] = &[
	("color_wheel",       "img/color_wheel.png"),
];

pub fn load_images(lua: &Lua) -> LuaResult<()> {
	let state = &mut *lua.app_data_mut::<State>().unwrap();

	let image = lua.create_table()?;

	let mut image_ids = Vec::new();

	for (index, (name, path)) in BUILTIN_IMG.iter().enumerate() {
		let bytes = Asset::get(path).unwrap().data;
		let image_id = state.canvas.load_image_mem(&bytes, ImageFlags::empty()).unwrap();

		image.set(*name, index)?;
		image_ids.push(image_id);
	}

	state.image_ids = image_ids;

	let tessera: LuaTable = lua.globals().get("tessera")?;
	tessera.set("image", image)?;

	Ok(())
}
