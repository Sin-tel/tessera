# Justidaw

***Warning: this is in a very early stage of development.***

Experimental DAW with a focus on microtonal composition, expressiveness and physical modelling.

GUI parts are written in lua with [LÖVE](https://love2d.org/), the audio backend is in Rust.

### How to build (Windows specific)
* Make sure you have installed [Rust](https://www.rust-lang.org/tools/install).
* You'll want ASIO support, detailed build instructions are on the [cpal repo](https://github.com/RustAudio/cpal).
* mlua needs to link to luajit. Get it from [here](https://github.com/LuaJIT/LuaJIT/tree/v2.1). Easiest way to build is to get MSVC, opening a "x64 Native Tools Command Prompt", cd to `luajit\src` and run `msvcbuild`.
* Set `LUA_INC`, `LUA_LIB`, `LUA_LIB_NAME` environment variables. (ex. `LUA_INC=C:\path\luajit\src`, `LUA_LIB=C:\path\luajit\src`, `LUA_LIB_NAME=lua51`)
* Build the rust library with `cargo build --release`.
* Install [LÖVE](https://love2d.org/).
* Run "love ." in the lua folder.

When things stabilize I will provide builds.
