# Tessera

<img src="screenshot.png" alt="syntax" style="width:763px;"/>

Experimental DAW with a focus on microtonal composition, expressiveness and physical modelling.

GUI parts are written in lua with [LÖVE](https://love2d.org/), the audio backend is in Rust.

### How to build
* Make sure you have installed [Rust](https://www.rust-lang.org/tools/install).
* For Windows builds, you'll want ASIO support. Detailed build instructions are on the [cpal repo](https://github.com/RustAudio/cpal#asio-on-windows).
* `cargo build` will build the backend.
* Install [LÖVE](https://love2d.org/).
* Run `love .` in the lua folder.

A setup file will automatically be generated in [lua/settings](lua/settings) where you can configure your audio/midi device, see [example_setup.lua](lua/settings/example_setup.lua) for instructions.

For release mode, use `cargo build --release` and set `release = true` in `main.lua`.

When things stabilize I will provide builds.
