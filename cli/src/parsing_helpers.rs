use crate::types::{Span, ParseError, ParseResultStr};


use nom::character::complete::alphanumeric1;



pub fn start_alpha<'a>(input: Span<'a>) -> ParseResultStr<'a> {
    let (input, first) = alphanumeric1(input)?;
    if first.fragment().starts_with(|c: char| c.is_alphabetic()) {
        Ok((input, first))
    } else {
        Err(nom::Err::Failure(ParseError::new("Expected alphabetic character", first)))
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