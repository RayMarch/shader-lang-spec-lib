use crate::nom_prelude::*;
use derive_deref::{Deref, DerefMut};

#[derive(Debug, Clone, Deref, PartialEq, Eq)]
pub struct Ident(String);

impl Ident {
    pub fn parse(s: &str) -> IResult<&str, Self> {
        let parser = |s| -> IResult<&str, &str> {
            recognize(pair(
                alt((alpha1, tag("_"))),
                many0(alt((alphanumeric1, tag("_")))),
            ))(s)
        };
        map(parser, |x| Ident(x.to_string()))(s)
    }
}

impl From<&str> for Ident {
    fn from(value: &str) -> Self {
        Ident(value.to_string())
    }
}

macro_rules! make_ty {
    ($name: ident) => {
        Ty { name: stringify!($name).into(), params: vec![], }
    };
    ($name: ident <$($param: ident),*>) => {
        Ty { name: stringify!($name).into(), params: vec![$(make_ty!($param)),*],}
    };
    ($name: ident <$($ty: expr,)*>) => {
        Ty { name: stringify!($name).into(), params: vec![$($ty),*],}
    };
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ty {
    pub name: Ident,
    pub params: Vec<Ty>,
}

impl Ty {
    pub fn parse(s: &str) -> IResult<&str, Ty> {
        let parser = tuple((
            Ident::parse,
            opt(delimited(
                ws0_then(tag("<")),
                ws0_then(separated_list0(ws0_then(tag(",")), ws0_then(Ty::parse))),
                ws0_then(tag(">")),
            )),
        ));
        map(parser, |(name, params)| Ty {
            name,
            params: params.unwrap_or_default(),
        })(s)
    }

    /// turns `None` to `void`, `Some(ty)` to `ty`
    pub fn flatten(ty: Option<Ty>) -> Ty {
        ty.unwrap_or(make_ty!(void))
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

impl FnDecl {
    pub fn parse(s: &str) -> IResult<&str, Self> {
        let parser = tuple((
            preceded(ws0_then(tag("fn")), ws1_then(Ident::parse)),
            ws0_then(delimited(
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
            )),
            map(
                opt(preceded(ws0_then(tag("->")), ws0_then(Ty::parse))),
                Ty::flatten,
            ),
        ));

        map(parser, |(name, args, out)| FnDecl { name, args, out })(s)
    }
}

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
    fn test_ty() {
        let ty = make_ty!(f32);
        assert_eq!(Ty::parse("f32"), Ok(("", ty)));

        let ty = make_ty!(vec4<f32>);
        assert_eq!(Ty::parse("vec4<f32>"), Ok(("", ty)));

        let ty = make_ty!(vec4<A, B, C, D>);
        assert_eq!(Ty::parse("vec4<A, B, C, D>"), Ok(("", ty)));

        let ty = make_ty!(vec4<A, B, C, D>);
        assert_eq!(Ty::parse("vec4  < A,B, C, D  >"), Ok(("", ty)));

        let ty = make_ty!(vec4<make_ty!(vec3<f32>),>);
        assert_eq!(Ty::parse("vec4<vec3<f32>>"), Ok(("", ty)));
    }

    #[test]
    fn test_fn_decl() {
        let decl = make_fn!(fn foo(a: x, b: y) -> void);
        assert_eq!(FnDecl::parse("fn foo(a: x, b: y)"), Ok(("", decl)));
    }
}
