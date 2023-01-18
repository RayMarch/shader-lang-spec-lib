#![allow(clippy::match_like_matches_macro)]
#![allow(unused)]
pub mod misc;
use misc::*;
use std::error::Error;

pub mod nom_prelude;
pub mod wgsl;

pub fn wgsl_download_and_parse() -> Result<wgsl::WgslSpec, Box<dyn Error>> {
    wgsl::WgslSpec::from_download()
}
