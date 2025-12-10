use std::{env, io};
use winresource::WindowsResource;

fn main() -> io::Result<()> {
    dbg!(env::var_os("CARGO_CFG_TARGET_OS"));
    if env::var_os("CARGO_CFG_TARGET_OS") == Some("windows".into()) {
        WindowsResource::new()
            .set_icon("assets/icon.ico")
            .compile()?;
    }
    Ok(())
}
