# Justidaw

***Warning: this is in a very early stage of development.***

Experimental DAW with a focus on microtonal composition, expressiveness and physical modelling.

GUI parts are written in lua with [LÖVE](https://love2d.org/), the audio engine is in Rust.



## How to build
* Make sure you have installed [Rust](https://www.rust-lang.org/tools/install) and [cbindgen](https://github.com/eqrion/cbindgen).
* On Windows you'll want ASIO support, detailed build instructions are on the [cpal repo](https://github.com/RustAudio/cpal).
* Install [LÖVE](https://love2d.org/).
* Build the rust library with "build.sh".
* Run "love ." in the lua folder.

When things stabilize I will provide builds.
