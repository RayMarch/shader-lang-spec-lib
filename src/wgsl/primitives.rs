use crate::nom_prelude::*;
use derive_deref::{Deref, DerefMut};

#[derive(Debug, Clone, Deref, DerefMut, PartialEq, Eq)]
pub struct Ident(pub String);

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
    name: Ident,
    params: Vec<Ty>,
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
