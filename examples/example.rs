#![allow(unused)]
extern crate shader_lang_spec_lib;
use std::{collections::HashSet, error::Error};

use lazy_static::__Deref;
use shader_lang_spec_lib::{
    wgsl::{
        overload_row::{Bound, BoundKind, UnionBound},
        ty::Ty,
    },
    *,
};

fn main() -> Result<(), Box<dyn Error>> {
    let wgsl_spec = wgsl::WgslSpec::from_download()?;

    /// iterate over all overload table rows in the document
    let mut names = HashSet::new();

    for (i, row) in wgsl_spec.overloads.iter().enumerate() {
        let mut rows = vec![row.clone()];
        let rows = row.instantiate_bounds_if_possible();
        for mut row in rows {
            //rename some Union bounds
            row.rename_type_params(|bound| {
                //println!("BOUND: {bound}");
                match &bound.bound_kind {
                    BoundKind::Union(x) => match &x[..] {
                        [a, b] => {
                            //println!("IU: {:?}", [a, b]);
                            let is_iuf32 =
                                [a, b] == [&Ty::try_from_str("i32")?, &Ty::try_from_str("u32")?];
                            return Some(Ty::try_from_str("iu32")?);
                        }
                        [a, b, c] => {
                            let is_iuf32 = [a, b, c]
                                == [
                                    &Ty::try_from_str("i32")?,
                                    &Ty::try_from_str("u32")?,
                                    &Ty::try_from_str("f32")?,
                                ];
                            return Some(Ty::try_from_str("iuf32")?);
                        }
                        _ => {}
                    },
                    _ => (),
                }
                None
            });

            let f = row.fn_decl;
            println!("{f}");

            for ty in f.tys_mentioned() {
                match ty.kind {
                    wgsl::ty::TyKind::Texture(_) => {
                        names.insert(ty.to_string());
                    }
                    _ => (),
                }
            }
        }
    }

    let mut vec: Vec<_> = names.iter().collect();
    vec.sort();
    println!("{:#?}", vec);

    // // textureDimensions is not instantiated correctly yet
    // let inst = wgsl_spec
    //     .overloads
    //     .iter()
    //     .find(|x| x.fn_decl.name.as_str() == "textureDimensions")
    //     .unwrap()
    //     .instantiate_bounds_if_possible();
    // for (i, f) in inst.iter().enumerate() {
    //     println!("{i}: {}", f);
    // }

    Ok(())
}

#[rustfmt::skip]
/// names of functions in example blocks within the wgsl spec
const EXCLUDE_NAMES: &[&'static str] = &["f", "f1", "f2", "f3", "f4", "f5", "f6", "f7", "f8", "bar", "baz", "foo", "fun", "bar2", "func", "main", "user", "scale", "caller", "nested", "simple", "sorter", "two_pi", "add_one", "add_two", "get_age", "my_func", "reverser", "shade_it", "shuffler", "bump_item", "comp_main", "float_fun", "vert_main", "bad_shader", "fragShader", "if_example", "advance_item", "missing_return", "switch_example", "discard_if_shallow", "precedence_example", "gather_x_components", "gather_y_components", "gather_z_components", "conditional_continue", "gather_depth_compare", "continue_out_of_loop", "invalid_infinite_loop", "gather_depth_components", "continue_end_of_loop_body", "increment_and_yield_previous", "redundant_continue_with_continuing"];
