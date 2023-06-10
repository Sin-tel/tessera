// #![allow(dead_code)]
// #![deny(clippy::pedantic)]
#![warn(clippy::cast_lossless)]
#![allow(clippy::excessive_precision)]

pub mod audio;
pub mod defs;
pub mod device;
pub mod dsp;
pub mod effect;
pub mod instrument;
pub mod lua;
pub mod pan;
pub mod render;
pub mod scope;