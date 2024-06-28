use crate::types::{Span, ParseResultStr};
use crate::error::ParseError;


use nom::character::complete::{alphanumeric1, space0};
use nom::bytes::complete::take_until;



pub fn start_alpha(input: Span) -> ParseResultStr {
    let (input, _) = space0(input)?;
    let (input, first) = alphanumeric1(input)?;
    if first.fragment().starts_with(|c: char| c.is_alphabetic()) {
        Ok((input, first))
    } else {
        Err(nom::Err::Error(ParseError::new("Expected alphabetic character", first)))
    }
}

pub fn mcut<I, O, F>(mut parser: F, message: &str) -> impl FnMut(I) -> nom::IResult<I, O, ParseError>
where
  F: nom::Parser<I, O, ParseError>,
{
    let message = message.to_string();
    move |input: I| match parser.parse(input) {
        Err(nom::Err::Error(mut e)) => {
            e.update_message(message.clone());
            Err(nom::Err::Failure(e))
        },
        rest => rest,
    }
}

pub fn prepend_cut<I, O, F>(mut parser: F, message: &str) -> impl FnMut(I) -> nom::IResult<I, O, ParseError>
where
  F: nom::Parser<I, O, ParseError>,
{
    let message = message.to_string();
    move |input: I| match parser.parse(input) {
        Err(nom::Err::Error(mut e)) => {
            e.prepend_message(message.clone());
            Err(nom::Err::Error(e))
        },
        rest => rest,
    }
}


pub fn trim<'a, F>(f: F) -> impl Fn(Span<'a>) -> ParseResultStr<'a>
where
    F: Fn(Span<'a>) -> ParseResultStr<'a> + 'a,
{
    move |input| {
        let (input, _) = space0(input)?;
        let (input, res) = f(input)?;
        let (input, _) = space0(input)?;
        Ok((input, res))
    }
}

pub fn unwrap(begin: &str, end: &str) -> impl Fn(Span) -> ParseResultStr
{
    let begin = begin.to_string();
    let end = end.to_string();
    move |input| {
        let (input, _) = tag(&begin)(input)?;
        let (input, res) = take_until(&end[..])(input)?;
        let (input, _) = tag(&end)(input)?;
        Ok((input, res))
    }
}

pub fn tag(s: &str) -> impl Fn(Span) -> ParseResultStr
{
    let s = s.bytes().collect::<Vec<u8>>();
    move |input| {
        let parser = nom::bytes::complete::tag(s.as_slice());
        let (input, res) = parser(input)?;
        Ok((input, res))
    }
}

pub fn safe_unwrap(begin: &str, end: &str) -> impl Fn(Span) -> (Span, Span)
{
    let begin = begin.to_string();
    let end = end.to_string();
    move |input| {
        let result: Result<(nom_locate::LocatedSpan<&str>, nom_locate::LocatedSpan<&str>), nom::Err<ParseError>> = unwrap(&begin, &end)(input);
        match result {
            Err(_) => (input, input),
            _ => result.unwrap()
        }
    }
}
