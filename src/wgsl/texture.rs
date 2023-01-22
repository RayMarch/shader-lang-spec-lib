use std::fmt::Display;

use nom::combinator::fail;

use crate::nom_prelude::*;

struct Sampler {
    /// name contains "comparision"
    comparision: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextureName {
    /// name contains "depth"
    depth: bool,
    /// name contains "storage"
    storage: bool,
    /// name contains "multisampled"
    multisampled: bool,
    /// dimensionality: "1d", "2d", "3d", or "cube"
    dimensionality: String,
    /// name contains "external"
    external: bool,
    /// name contains "array"
    array: bool,
}

impl TextureName {
    pub fn parse(s: &str) -> NomResult<&str, TextureName> {
        fn opt_attrib<'a>(att: &'a str) -> impl FnMut(&'a str) -> NomResult<&'a str, bool> {
            map(opt(tag(att)), |x| x.is_some())
        }

        let dims = preceded(
            tag("_"),
            alt((
                tag("1d"),
                tag("2d"),
                tag("3d"),
                tag("cube"),
                tag("external"), //external implies 2d
            )),
        );

        let parser = preceded(
            tag("texture"),
            tuple((
                opt_attrib("_depth"),
                opt_attrib("_storage"),
                opt_attrib("_multisampled"),
                dims,
                opt_attrib("_array"),
            )),
        );

        map(parser, |(depth, storage, multisampled, dims, array)| {
            TextureName {
                depth,
                storage,
                array,
                multisampled,
                external: dims == "external",
                dimensionality: if dims == "external" { "2d" } else { dims }.to_string(),
            }
        })(s)
    }
}

impl Display for TextureName {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "texture")?;
        if self.depth {write!(f, "_depth")?;}
        if self.storage {write!(f, "_storage")?;}
        if self.multisampled {write!(f, "_multisampled")?;}
        match self.external {
            true => write!(f, "_external")?,
            false => write!(f, "_{}", self.dimensionality)?,
        }
        if self.array {write!(f, "_array")?;}
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
            (let $s: ident = $e: expr $(;)?) => {
                assert_eq!(TextureName::parse(std::stringify!($s)), Ok(("", $e)))
            };
        }
        macro_rules! fail {
            ($s: ident) => {
                assert!(TextureName::parse(std::stringify!($s)).is_err())
            };
        }
        ok!(
            let texture_2d_array = TextureName {
                array: true,
                dimensionality: "2d".to_string(),
                ..rest
            }
        );

        ok!(
            let texture_depth_2d_array = TextureName {
                array: true,
                depth: true,
                dimensionality: "2d".to_string(),
                ..rest
            };
        );

        ok!(
            let texture_cube = TextureName {
                dimensionality: "cube".to_string(),
                ..rest
            };
        );

        ok!(
            let texture_external = TextureName {
                dimensionality: "2d".to_string(),
                external: true,
                ..rest
            };
        );
        fail!(texture_depth_storage_depth);
        fail!(depth_storage_depth);
        fail!(texture_x_2d_array);
    }
}
