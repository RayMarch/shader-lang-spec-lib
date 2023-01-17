pub use nom::branch::alt;
pub use nom::bytes::complete::take_till;
pub use nom::bytes::complete::{tag, take_while, take_while1};
pub use nom::character::complete::{alpha1, alphanumeric1, digit1, newline};
pub use nom::combinator::cut;
pub use nom::combinator::map;
pub use nom::combinator::opt;
pub use nom::combinator::peek;
pub use nom::error::context;
pub use nom::error::convert_error;
pub use nom::multi::many0;
pub use nom::multi::separated_list0;
pub use nom::multi::separated_list1;
pub use nom::sequence::*;
pub use nom::*;
pub use nom::{combinator::recognize, multi::many0_count, sequence::pair};

pub type NomError<I> = nom::error::VerboseError<I>;

pub type NomResult<I, O, E = NomError<I>> = Result<(I, O), Err<E>>;

pub fn identifier(input: &str) -> NomResult<&str, &str> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0_count(alt((alphanumeric1, tag("_")))),
    ))(input)
}

pub fn ws1(s: &str) -> NomResult<&str, &str> {
    take_while1(char::is_whitespace)(s)
}

pub fn ws0(s: &str) -> NomResult<&str, &str> {
    take_while(char::is_whitespace)(s)
}

pub fn ws1_then<'a, O, F>(mut f: F) -> impl FnMut(&'a str) -> NomResult<&str, O, NomError<&str>>
where
    F: Parser<&'a str, O, NomError<&'a str>>,
{
    move |input: &str| {
        let (input, _) = ws1.parse(input)?;
        f.parse(input).map(|(i, o)| (i, o))
    }
}

pub fn ws0_then<'a, O, F>(mut f: F) -> impl FnMut(&'a str) -> NomResult<&str, O, NomError<&str>>
where
    F: Parser<&'a str, O, NomError<&'a str>>,
{
    move |input: &str| {
        let (input, _) = ws0.parse(input)?;
        f.parse(input).map(|(i, o)| (i, o))
    }
}

/// debug helper
pub trait NomReportError {
    type Input;
    type Output;
    fn report(self, input: Self::Input) -> Self;
}

impl<I: core::ops::Deref<Target = str>, O: std::fmt::Debug> NomReportError
    for NomResult<I, O, NomError<I>>
{
    type Input = I;
    type Output = Self;
    fn report(self, input: Self::Input) -> Self {
        match self.is_ok() {
            true => {
                match &self {
                    Ok((i, o)) => println!("success: {o:#?}"),
                    Err(_) => unreachable!(),
                };
                self
            }
            false => match self {
                Ok(_) => unreachable!(),
                Result::Err(e) => {
                    let e: Err<NomError<I>> = e;
                    e.report(input);
                    unreachable!()
                }
            },
        }
    }
}

impl<I: core::ops::Deref<Target = str>> NomReportError for Err<NomError<I>> {
    type Input = I;
    type Output = Self;
    fn report(self, input: Self::Input) -> Self {
        let text = match self {
            Err::Incomplete(e) => unreachable!(),
            Err::Error(e) | Err::Failure(e) => convert_error(input, e),
        };
        panic!("{text}")
    }
}

// debug helper
// pub trait NomReportParse<O> {
//     type Input<'a>;
//     fn report<'a>(self, input: Self::Input<'a>) -> Self;
// }

// impl<O, F> NomReportParse<O> for F where F: for<'a> Parser<&'a str, O, NomError<&'a str>> {
//     type Input = &'a str;
//     type Output = IResult<I, O, E>;
// }
