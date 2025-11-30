// image.rs
use crate::app::State;
use femtovg::ImageFlags;
use mlua::prelude::*;

#[rustfmt::skip]
pub const BUILTIN_ICONS: &[(&str, &[u8])] = &[
	("solo",       include_bytes!("../../assets/icons/solo.png")),
	("mute",       include_bytes!("../../assets/icons/mute.png")),
	("armed",      include_bytes!("../../assets/icons/armed.png")),
	("visible",    include_bytes!("../../assets/icons/visible.png")),
	("invisible",  include_bytes!("../../assets/icons/invisible.png")),
	("lock",       include_bytes!("../../assets/icons/lock.png")),
	("unlock",     include_bytes!("../../assets/icons/unlock.png")),
];

pub fn load_images(lua: &Lua) -> LuaResult<()> {
	let state = &mut *lua.app_data_mut::<State>().unwrap();

	let image = lua.create_table()?;

	let mut image_ids = Vec::new();

	for (index, (name, bytes)) in BUILTIN_ICONS.iter().enumerate() {
		let image_id = state.canvas.load_image_mem(bytes, ImageFlags::empty()).unwrap();

		image.set(*name, index)?;
		image_ids.push(image_id);
	}

	state.image_ids = image_ids;

	let tessera: LuaTable = lua.globals().get("tessera")?;
	tessera.set("image", image)?;

	Ok(())
}
