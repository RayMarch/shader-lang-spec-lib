use std::fmt::Display;

use nom::combinator::fail;

use crate::nom_prelude::*;

struct Sampler {
    /// name contains "comparision"
    comparision: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextureName {
    /// name contains "depth"
    depth: bool,
    /// name contains "storage"
    storage: bool,
    /// name contains "array"
    array: bool,
    /// name contains "multisampled"
    multisampled: bool,
    /// name contains "external"
    external: bool,
    /// dimensionality: "1d", "2d", "3d", or "cube"
    dimensionality: String,
}

impl TextureName {
    pub fn parse(s: &str) -> NomResult<&str, TextureName> {
        // texture_2d<ST>
        // texture_2d_array<ST>
        // texture_cube<ST>
        // texture_cube_array<ST>
        // texture_multisampled_2d<ST>
        // texture_depth_2d
        // texture_depth_2d_array
        // texture_depth_cube
        // texture_depth_cube_array
        // texture_depth_multisampled_2d
        // texture_storage_2d<F,A>
        // texture_storage_2d_array<F,A>
        // texture_external

        // external > depth > storage > ms > dim > array

        fn opt_attrib<'a>(att: &'a str) -> impl FnMut(&'a str) -> NomResult<&'a str, bool> {
            map(opt(tag(att)), |x| x.is_some())
        }

        let dims = preceded(
            tag("_"),
            alt((tag("1d"), tag("2d"), tag("3d"), tag("cube"))),
        );

        let parser = preceded(
            tag("texture"),
            tuple((
                opt_attrib("_external"),
                opt_attrib("_depth"),
                opt_attrib("_storage"),
                opt_attrib("_multisampled"),
                dims,
                opt_attrib("_array"),
            )),
        );

        map(
            parser,
            |(external, depth, storage, multisampled, dims, array)| TextureName {
                depth,
                storage,
                array,
                multisampled,
                external,
                dimensionality: dims.to_string(),
            },
        )(s)
    }
}

impl Display for TextureName {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "texture")?;
        if self.depth {write!(f, "_depth")?;}
        if self.storage {write!(f, "_storage")?;}
        write!(f, "_{}", self.dimensionality)?;
        if self.array {write!(f, "_array")?;}
        if self.external {write!(f, "_external")?;}
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    pub use super::*;

    #[test]
    fn test_texture_name() {
        let rest = TextureName {
            depth: false,
            storage: false,
            array: false,
            multisampled: false,
            external: false,
            dimensionality: "".to_string(),
        };

        macro_rules! ok {
            ($s: ident) => {
                assert_eq!(TextureName::parse(std::stringify!($s)), Ok(("", $s)))
            };
        }
        macro_rules! fail {
            ($s: literal) => {
                assert!(Ident::parse($s).is_err())
            };
        }

        let texture_2d_array = TextureName {
            array: true,
            dimensionality: "2d".to_string(),
            ..rest
        };
        ok!(texture_2d_array);

        let texture_depth_2d_array = TextureName {
            array: true,
            depth: true,
            dimensionality: "2d".to_string(),
            ..rest
        };
        ok!(texture_depth_2d_array);

        let texture_cube = TextureName {
            dimensionality: "cube".to_string(),
            ..rest
        };
        ok!(texture_cube);

        // texture_2d
        // texture_2d_array
        // texture_cube
        // texture_cube_array
        // texture_multisampled_2d
        // texture_depth_2d
        // texture_depth_2d_array
        // texture_depth_cube
        // texture_depth_cube_array
        // texture_depth_multisampled_2d
        // texture_storage_2d
        // texture_storage_2d_array
        // texture_external
    }
}
