use std::{env, io};
use winresource::WindowsResource;

fn main() -> io::Result<()> {
	println!("cargo:rerun-if-changed=build.rs");
	println!("cargo:rerun-if-changed=assets/icon.ico");

	// Only run on Windows builds
	if env::var_os("CARGO_CFG_TARGET_OS") == Some("windows".into()) {
		WindowsResource::new().set_icon("assets/icon.ico").compile()?;
	}
	Ok(())
}
