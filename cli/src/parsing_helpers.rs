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


pub fn cut_with_message<'a, T: std::fmt::Debug>(parsed: Result<T, nom::Err<ParseError<'a>>>, message: &str) -> Result<T, nom::Err<ParseError<'a>>> {
    match parsed {
        Ok(resp) => Ok(resp),
        Err(nom::Err::Error(mut e)) | Err(nom::Err::Failure(mut e)) => {
            e.update_message(message);
            Err(nom::Err::Failure(e))
        },
        k => panic!("Unexpected parse error {:?}", k),
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
