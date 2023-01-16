use crate::nom_prelude::*;
use derive_deref::{Deref, DerefMut};

#[derive(Debug, Clone, Deref, DerefMut)]
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

#[derive(Debug, Clone)]
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
