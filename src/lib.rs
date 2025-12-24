#![deny(unreachable_patterns)]
#![warn(clippy::cast_lossless)]
#![warn(clippy::uninlined_format_args)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![warn(clippy::explicit_iter_loop)]
#![warn(clippy::ignored_unit_patterns)]
#![warn(clippy::redundant_else)]
#![warn(clippy::match_same_arms)]
#![warn(clippy::single_match_else)]
#![warn(clippy::unnested_or_patterns)]
#![warn(clippy::inconsistent_struct_constructor)]
#![warn(clippy::unused_self)]
#![warn(clippy::needless_borrow)]
#![warn(clippy::match_wildcard_for_single_variants)]
#![warn(clippy::manual_assert)]
#![warn(clippy::manual_let_else)]
#![warn(clippy::unnecessary_semicolon)]
#![warn(clippy::large_stack_arrays)]
#![allow(clippy::match_like_matches_macro)]
#![allow(clippy::enum_variant_names)]
#![allow(clippy::new_without_default)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::get_first)]
#![allow(clippy::too_many_arguments)]

pub mod audio;
mod channel;
pub mod context;
mod effect;
pub mod embed;
pub mod engine;
mod instrument;
mod meters;
pub mod midi;
mod render;
mod scope;
mod voice_manager;
mod worker;

#[allow(dead_code)]
pub mod dsp;

#[allow(unused_macros)]
#[allow(unused_imports)]
pub mod log;

pub mod api;
pub mod app;
pub mod opengl;
mod text;
