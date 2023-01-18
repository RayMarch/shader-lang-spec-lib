pub use nom;
pub use nom::branch::alt;
pub use nom::bytes::complete::take_till;
use nom::bytes::complete::take_until;
pub use nom::bytes::complete::{tag, take_while, take_while1};
pub use nom::character::complete::{alpha1, alphanumeric1, digit1, newline};
pub use nom::combinator::cut;
pub use nom::combinator::eof;
use nom::combinator::fail;
pub use nom::combinator::map;
pub use nom::combinator::opt;
pub use nom::combinator::peek;
pub use nom::error::context;
pub use nom::error::convert_error;
use nom::error::ParseError;
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
    fn report_into_string(self, input: Self::Input) -> String;
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
    fn report_into_string(self, input: Self::Input) -> String {
        match self.is_ok() {
            true => match &self {
                Ok((i, o)) => "success".to_string(),
                Err(_) => unreachable!(),
            },
            false => match self {
                Ok(_) => unreachable!(),
                Result::Err(e) => {
                    let e: Err<NomError<I>> = e;
                    e.report_into_string(input)
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
    fn report_into_string(self, input: Self::Input) -> String {
        match self {
            Err::Incomplete(e) => unreachable!(),
            Err::Error(e) | Err::Failure(e) => convert_error(input, e),
        }
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

// pub fn tag2<T, Input, Error: ParseError<Input>>(
//     tag: T,
// ) -> impl Fn(Input) -> IResult<Input, Input, Error>
// where
//     Input: InputTake + Compare<T>,
//     T: InputLength + Clone,
// {
//     move |i: Input| {
//         let tag_len = tag.input_len();
//         let t = tag.clone();
//         let res: IResult<_, _, Error> = match i.compare(t) {
//             CompareResult::Ok => Ok(i.take_split(tag_len)),
//             _ => {
//                 let e: nom::error::ErrorKind = nom::error::ErrorKind::Tag;
//                 Err(Err::Error(Error::from_error_kind(i, e)))
//             }
//         };
//         res
//     }
// }

pub fn take_until_matches<'a, F, O>(mut f: F) -> impl FnMut(&'a str) -> NomResult<&'a str, &'a str>
where
    F: Parser<&'a str, O, NomError<&'a str>>,
{
    move |i: &'a str| {
        let taken = i.char_indices().find_map(|(pos, _)| {
            let (taken, rest) = i.split_at(pos);
            f.parse(rest).ok().map(|_| taken)
        });

        match taken {
            Some(taken) => tag(taken)(i),
            None => fail(i),
        }
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_take_until_matches() {
        let str = "asdf<end>";
        assert_eq!(
            take_until_matches(tag("<end>"))(str).report(str),
            Ok(("<end>", "asdf"))
        );
    }
}

// pub fn take_until_matches<F, O, T, Input, Error: ParseError<Input>>(
//     f: F,
// ) -> impl Fn(Input) -> IResult<Input, Input, Error>
// where
//     F: for<'a> Parser<&'a str, O, NomError<&'a str>>,
//     Input: InputTake + Compare<T>,
//     T: InputLength + Clone,
// {
//     move |i: Input| {
//         let tag_len = tag.input_len();
//         let t = tag.clone();
//         let res: IResult<_, _, Error> = match i.compare(t) {
//             CompareResult::Ok => Ok(i.take_split(tag_len)),
//             _ => {
//                 let e: nom::error::ErrorKind = nom::error::ErrorKind::Tag;
//                 Err(Err::Error(Error::from_error_kind(i, e)))
//             }
//         };
//         res
//     }
// }
