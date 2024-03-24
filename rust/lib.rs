// #![allow(dead_code)]

#![warn(clippy::cast_lossless)]
#![warn(clippy::uninlined_format_args)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![warn(clippy::explicit_iter_loop)]
#![warn(clippy::items_after_statements)]
#![deny(unreachable_patterns)]
//
// #![deny(clippy::pedantic)]
// #![allow(clippy::cast_precision_loss)]
// #![allow(clippy::cast_possible_truncation)]
// #![allow(clippy::cast_sign_loss)]
// #![allow(clippy::cast_possible_wrap)]
// #![allow(clippy::similar_names)]
// #![allow(clippy::excessive_precision)]
// #![allow(clippy::unreadable_literal)]
// #![allow(clippy::wildcard_imports)]
// #![allow(clippy::too_many_lines)]
// #![allow(clippy::missing_panics_doc)]
// #![allow(clippy::missing_errors_doc)]
// #![allow(clippy::must_use_candidate)]
// #![allow(clippy::enum_glob_use)]
// #![allow(clippy::single_match_else)]

mod audio;
mod effect;
mod instrument;
mod lua;
mod render;
mod scope;

#[allow(dead_code)]
mod dsp;

#[allow(unused_macros)]
#[allow(unused_imports)]
mod log;
