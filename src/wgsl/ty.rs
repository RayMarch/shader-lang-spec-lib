use std::fmt::Display;

use nom::bytes::complete::take_until;
use nom::character::complete::anychar;

use super::fn_decl::Ident;
use super::texture::*;
use crate::nom_prelude::*;

#[macro_export]
macro_rules! make_ty {
    ($name: ident) => {
        Ty::new(std::stringify!($name).into(), vec![])
    };
    ($name: ident <$($param: ident),*>) => {
        Ty::new(std::stringify!($name).into(), vec![$(make_ty!($param),)*])
    };
    ($name: ident <$($ty: expr,)*>) => {
        Ty::new(std::stringify!($name).into(), vec![$($ty),*])
    };
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TyKind {
    /// return type of a function that returns nothing
    Void,
    /// i.e. vec4<_>, vecN<_>
    Vector(char),
    /// i.e. mat4x4<_>, matCxR<_>, mat2x3<_>, matCxC<_>
    Matrix(char, char),
    /// texture types with different parametrizations (2d, 3d, cube, depth...)
    Texture(TextureName),
    /// neither of the above
    Named(Ident),
}

impl Display for TyKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TyKind::Void => write!(f, "void"),
            TyKind::Vector(x) => write!(f, "vec{x}"),
            TyKind::Matrix(x, y) => write!(f, "mat{x}x{y}"),
            TyKind::Texture(t) => write!(f, "{t}"),
            TyKind::Named(ident) => write!(f, "{ident}"),
        }
    }
}

impl TyKind {
    pub fn parse(s: &str) -> NomResult<&str, TyKind> {
        alt((
            map(tag("void"), |s: &str| TyKind::Void),
            map(preceded(tag("vec"), cut(anychar)), TyKind::Vector),
            map(
                preceded(tag("mat"), cut(separated_pair(anychar, tag("x"), anychar))),
                |(x, y)| TyKind::Matrix(x, y),
            ),
            map(TextureName::parse, TyKind::Texture),
            map(Ident::parse, TyKind::Named),
        ))(s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ty {
    /// part before the angle brackets, if there are any
    pub kind: TyKind,
    /// type parameters in angle brackets.
    params: Vec<Ty>,
}

impl Ty {
    pub fn new(name: Ident, params: Vec<Ty>) -> Self {
        let (s, kind) =
            TyKind::parse(name.as_str()).expect("infallible because of `Ident` variant fallback");
        assert!(s.is_empty());
        Ty { kind, params }
    }

    pub fn parse(s: &str) -> NomResult<&str, Ty> {
        let parser = tuple((
            TyKind::parse,
            opt(delimited(
                ws0_then(tag("<")),
                ws0_then(separated_list0(ws0_then(tag(",")), ws0_then(Ty::parse))),
                ws0_then(tag(">")),
            )),
        ));
        map(parser, |(kind, params)| Ty {
            kind,
            params: params.unwrap_or_default(),
        })(s)
    }

    /// traverse the type and its type params, and transitive type params to find `t`
    pub fn find_mut(&mut self, t: &Ty) -> Option<&mut Ty> {
        match self == t {
            true => Some(self),
            _ => self.params.iter_mut().find_map(|p| p.find_mut(t)),
        }
    }

    /// traverse the type and its type params, and transitive type params to find `t`
    pub fn find(&self, t: &Ty) -> Option<&Ty> {
        match self == t {
            true => Some(self),
            _ => self.params.iter().find_map(|p| p.find(t)),
        }
    }

    /// turns `None` to `void`, `Some(ty)` to `ty`
    pub fn flatten(ty: Option<Ty>) -> Ty {
        ty.unwrap_or_else(|| make_ty!(void))
    }
}

impl Display for Ty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)?;
        if !self.params.is_empty() {
            write!(f, "<")?;
            for (i, ty) in self.params.iter().enumerate() {
                let comma = if i + 1 != self.params.len() { ", " } else { "" };
                write!(f, "{ty}{comma}")?;
            }
            write!(f, ">")?;
        }
        Ok(())
    }
}

impl From<Ident> for Ty {
    fn from(name: Ident) -> Self {
        Ty::new(name, vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ty() {
        let ty = make_ty!(f32);
        assert_eq!(Ty::parse("f32"), Ok(("", ty)));

        let ty = make_ty!(vec4<f32>);
        assert_eq!(Ty::parse("vec4<f32>"), Ok(("", ty)));

        let ty = make_ty!(vec4<A, B, C, D>);
        assert_eq!(Ty::parse("vec4<A, B, C, D>"), Ok(("", ty)));

        let ty = make_ty!(matCxR<A, B, C, D>);
        assert_eq!(Ty::parse("matCxR<A, B, C, D>"), Ok(("", ty)));

        let ty = make_ty!(vec4<A, B, C, D>);
        assert_eq!(Ty::parse("vec4  < A,B, C, D  >"), Ok(("", ty)));

        let ty = make_ty!(vec4<make_ty!(vec3<f32>),>);
        assert_eq!(Ty::parse("vec4<vec3<f32>>"), Ok(("", ty)));
    }
}
