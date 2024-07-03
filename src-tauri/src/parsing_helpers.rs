use crate::types::{Span, BaseParseResult, ParseResultStr};
use crate::error::ParseError;


use nom::character::complete::{alphanumeric1, space0};
use nom::branch::alt;
use nom::sequence::delimited;



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


pub fn trim<'a, O, F>(f: F) -> impl FnMut(Span<'a>) -> BaseParseResult<'a, O>
    where F: FnMut(Span<'a>) -> BaseParseResult<'a, O> + 'a,
{
    delimited(alt((tag("\\ "), space0)), f, alt((tag("\\ "), space0)))
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
