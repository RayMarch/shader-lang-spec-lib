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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ty {
    name: Ident,
    params: Vec<Ty>,
}

impl Ty {
    pub fn parse(s: &str) -> IResult<&str, Ty> {
        let parser = tuple((
            Ident::parse,
            delimited(
                ws0_then(tag("<")),
                ws0_then(separated_list0(ws0_then(tag(",")), ws0_then(Ty::parse))),
                ws0_then(tag(">")),
            ),
        ));
        map(parser, |(name, params)| Ty { name, params })(s)
    }
}

#[test]
fn test_ty() {
    fn parser<'a>(s: &'a str) -> IResult<&'a str, Vec<&'a str>> {
        delimited(
            tag("<"),
            separated_list0(ws0_then(tag(",")), ws0_then(tag("asdf"))),
            tag(">"),
        )(s)
    }

    let ident = Ident("f32".to_string());
    assert_eq!(Ident::parse("f32>"), Ok((">", ident)));
    assert_eq!(parser("<asdf>"), Ok(("", vec!["asdf"])));

    let ty: Ty = Ty {
        name: "vec4".into(),
        params: vec![Ty {
            name: "u32".into(),
            params: vec![],
        }],
    };
    assert_eq!(Ty::parse("vec4<u32>"), Ok(("", ty)));
}
