use crate::make_ty;
use std::fmt::Display;

use crate::nom_prelude::*;
use derive_deref::{Deref, DerefMut};

use super::overload_row::*;
use super::ty::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ident(String);

impl Display for Ident {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl std::ops::Deref for Ident {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

impl Ident {
    pub fn parse(s: &str) -> NomResult<&str, Self> {
        let parser = |s| -> NomResult<&str, &str> {
            recognize(pair(
                alt((alpha1, tag("_"))),
                many0(alt((alphanumeric1, tag("_")))),
            ))(s)
        };
        map(parser, |x| x.into())(s)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for Ident {
    fn from(value: &str) -> Self {
        Ident(value.to_string())
    }
}

macro_rules! make_fn {
    (fn $name: ident ($($arg: ident: $arg_ty: ident),*) -> $out: ident) => {
        make_fn!(fn $name ($($arg: make_ty!($arg_ty)),*) -> make_ty!($out))
    };
    (fn $name: ident ($($arg: ident: $arg_ty: expr),*) -> $out: expr) => {
        FnDecl {
            name: stringify!($name).into(),
            args: vec![$((stringify!($arg).into(), $arg_ty)),*],
            out: $out,
        }
    };
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FnDecl {
    // fn foo(a: vec3<f32>, b: vec4<f32>) -> vec3<f32>
    pub name: Ident,
    pub args: Vec<(Ident, Ty)>,
    pub out: Ty,
}

impl Display for FnDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "fn {}(", self.name)?;
        let indent = "    ";
        let args_max = self
            .args
            .iter()
            .map(|(i, _)| i.len())
            .max()
            .unwrap_or_default();
        for (i, (arg, ty)) in self.args.iter().enumerate() {
            let comma = if i + 1 != self.args.len() { "," } else { "" };
            write!(f, "{indent}{} ", *arg)?;
            for _ in arg.len()..args_max {
                write!(f, " ")?;
            }
            writeln!(f, ": {ty}{comma}")?;
        }
        writeln!(f, ") -> {}", self.out)?;
        Ok(())
    }
}

impl FnDecl {
    pub fn parse(s: &str) -> NomResult<&str, Self> {
        let parser = tuple((
            preceded(tag("fn"), ws1_then(Ident::parse)),
            cut(ws0_then(delimited(
                ws0_then(tag("(")),
                ws0_then(separated_list0(
                    ws0_then(tag(",")),
                    separated_pair(
                        ws0_then(Ident::parse),
                        ws0_then(tag(":")),
                        ws0_then(Ty::parse),
                    ),
                )),
                ws0_then(tag(")")),
            ))),
            map(
                opt(preceded(ws0_then(tag("->")), ws0_then(Ty::parse))),
                Ty::flatten,
            ),
        ));

        map(parser, |(name, args, out)| FnDecl { name, args, out })(s)
    }

    pub fn tys_mentioned(&self) -> impl Iterator<Item = &Ty> + Clone {
        std::iter::once(&self.out).chain(self.args.iter().map(|(_, ty)| ty))
    }

    pub fn tys_mentioned_mut(&mut self) -> impl Iterator<Item = &mut Ty> {
        std::iter::once(&mut self.out).chain(self.args.iter_mut().map(|(_, ty)| ty))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_ident() {
        macro_rules! ok {
            ($s: literal) => {
                assert_eq!(Ident::parse($s), Ok(("", $s.into())))
            };
        }
        macro_rules! fail {
            ($s: literal) => {
                assert!(Ident::parse($s).is_err())
            };
        }
        ok!("f32");
        ok!("_1");
        ok!("a_582_");
        ok!("_582_");
        ok!("yeet");
        fail!("1");
        fail!("");
        fail!("4_");
        fail!("123");
        fail!("<f32>");
        fail!("$");
        fail!("\n");
        fail!(" ");
    }

    #[test]
    fn test_fn_decl() {
        let decl = make_fn!(fn foo(a: x, b: y) -> void);
        assert_eq!(FnDecl::parse("fn foo(a: x, b: y)"), Ok(("", decl)));

        let decl = make_fn!(fn foo(a: x, b: y) -> f32);
        assert_eq!(FnDecl::parse("fn foo(a: x, b: y) -> f32"), Ok(("", decl)));

        let decl = make_fn!(fn foo() -> f32);
        assert_eq!(FnDecl::parse("fn foo() -> f32"), Ok(("", decl)));

        let decl = make_fn!(fn foo() -> void);
        assert_eq!(FnDecl::parse("fn foo()"), Ok(("", decl)));

        let decl = make_fn!(fn foo() -> void);
        assert!(FnDecl::parse("fn ()").is_err());
    }
}
