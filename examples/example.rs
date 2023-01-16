#![allow(unused)]
extern crate shader_lang_spec_lib;
use std::error::Error;

use shader_lang_spec_lib::*;

fn main() -> Result<(), Box<dyn Error>> {
    let wgsl = wgsl::WgslSpec::from_download()?;
    Ok(())
}
