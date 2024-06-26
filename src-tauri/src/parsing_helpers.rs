use crate::types::{Span, ParseError, ParseResultStr};


use nom::character::complete::{alphanumeric1, space0, char};
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

pub fn mcut<'a, I, O, F>(mut parser: F, message: &'a str) -> impl FnMut(I) -> nom::IResult<I, O, ParseError<'a>>
where
  F: nom::Parser<I, O, ParseError<'a>>,
{
  move |input: I| match parser.parse(input) {
    Err(nom::Err::Error(mut e)) => {
        e.update_message(message);
        Err(nom::Err::Failure(e))
    },
    rest => rest,
  }
}

pub fn prepend_cut<'a, I, O, F>(mut parser: F, message: &'a str) -> impl FnMut(I) -> nom::IResult<I, O, ParseError<'a>>
where
  F: nom::Parser<I, O, ParseError<'a>>,
{
  move |input: I| match parser.parse(input) {
    Err(nom::Err::Error(mut e)) => {
        e.prepend_message(message);
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

pub fn unwrap(begin: char, end: char) -> impl Fn(Span) -> ParseResultStr
{
    move |input| {
        let (input, _) = char(begin)(input)?;
        let (input, res) = take_until(&end.to_string()[..])(input)?;
        let (input, _) = char(end)(input)?;
        Ok((input, res))
    }
}

pub fn safe_unwrap(begin: char, end: char) -> impl Fn(Span) -> (Span, Span)
{
    move |input| {
        let result = unwrap(begin, end)(input);
        match result {
            Err(_) => (input, input),
            _ => result.unwrap()
        }
    }
}
