pub use nom::branch::alt;
pub use nom::bytes::complete::{tag, take_while, take_while1};
pub use nom::character::complete::{alpha1, alphanumeric1, digit1, newline};
pub use nom::combinator::map;
pub use nom::error::*;
pub use nom::multi::many0;
pub use nom::multi::separated_list0;
pub use nom::sequence::*;
pub use nom::*;
pub use nom::{combinator::recognize, multi::many0_count, sequence::pair};

pub type VResult<I, O, E = error::VerboseError<I>> = Result<(I, O), Err<E>>;

pub fn identifier(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0_count(alt((alphanumeric1, tag("_")))),
    ))(input)
}

pub fn ws1(s: &str) -> IResult<&str, &str> {
    take_while1(char::is_whitespace)(s)
}

pub fn ws0(s: &str) -> IResult<&str, &str> {
    take_while(char::is_whitespace)(s)
}

pub fn ws0_then<'a, O, F>(mut f: F) -> impl FnMut(&'a str) -> IResult<&str, O, error::Error<&str>>
where
    F: Parser<&'a str, O, error::Error<&'a str>>,
{
    move |input: &str| {
        let (input, _) = ws0.parse(input)?;
        f.parse(input).map(|(i, o)| (i, o))
    }
}
