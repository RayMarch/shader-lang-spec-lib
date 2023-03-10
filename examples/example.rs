#![allow(unused)]
extern crate shader_lang_spec_lib;
use std::{collections::HashSet, error::Error};

use lazy_static::__Deref;
use shader_lang_spec_lib::*;

fn main() -> Result<(), Box<dyn Error>> {
    let wgsl_spec = wgsl::WgslSpec::from_download()?;

    /// iterate over all overload table rows in the document
    for f in wgsl_spec.overloads {
        println!("{}", f);
    }
    Ok(())
}

const EXCLUDE_NAMES: &[&'static str] = &[
    "f",
    "f1",
    "f2",
    "f3",
    "f4",
    "f5",
    "f6",
    "f7",
    "f8",
    "bar",
    "baz",
    "foo",
    "fun",
    "bar2",
    "func",
    "main",
    "user",
    "scale",
    "caller",
    "nested",
    "simple",
    "sorter",
    "two_pi",
    "add_one",
    "add_two",
    "get_age",
    "my_func",
    "reverser",
    "shade_it",
    "shuffler",
    "bump_item",
    "comp_main",
    "float_fun",
    "vert_main",
    "bad_shader",
    "fragShader",
    "if_example",
    "advance_item",
    "missing_return",
    "switch_example",
    "discard_if_shallow",
    "precedence_example",
    "gather_x_components",
    "gather_y_components",
    "gather_z_components",
    "conditional_continue",
    "gather_depth_compare",
    "continue_out_of_loop",
    "invalid_infinite_loop",
    "gather_depth_components",
    "continue_end_of_loop_body",
    "increment_and_yield_previous",
    "redundant_continue_with_continuing",
];
