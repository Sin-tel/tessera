// #![allow(dead_code)]

#![deny(unreachable_patterns)]
#![warn(clippy::cast_lossless)]
#![warn(clippy::uninlined_format_args)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![warn(clippy::explicit_iter_loop)]
#![warn(clippy::items_after_statements)]
#![warn(clippy::ignored_unit_patterns)]
#![warn(clippy::redundant_else)]
#![warn(clippy::match_same_arms)]
#![warn(clippy::single_match_else)]
#![allow(clippy::match_like_matches_macro)]
#![allow(clippy::enum_variant_names)]
#![allow(clippy::new_without_default)]
#![warn(clippy::unnested_or_patterns)]
#![warn(clippy::inconsistent_struct_constructor)]
#![warn(clippy::unused_self)]
#![warn(clippy::needless_borrow)]
#![warn(clippy::match_wildcard_for_single_variants)]
#![warn(clippy::manual_assert)]
#![warn(clippy::manual_let_else)]

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
// #![allow(clippy::struct_field_names)]
// #![allow(clippy::module_name_repetitions)]

mod audio;
mod effect;
mod instrument;
mod lua;
mod midi;
mod render;
mod scope;

#[allow(dead_code)]
// needs to be public for benches
pub mod dsp;

#[allow(unused_macros)]
#[allow(unused_imports)]
mod log;
