use crate::types::Span;
use thiserror::Error;
use serde::Serialize;
use nom;


#[derive(Error, Debug, Serialize)]
pub enum Error {
    #[error("unable to parse input: {}", .source.span.fragment)]
    ParseError{
        #[from]
        source: ParseError
    },
    #[error("evaluation error: {0}")]
    EvalError(String),
    #[error("unit error: {0}")]
    UnitError(String),
}


#[derive(Debug, Serialize, PartialEq)]
pub struct ParseError {
    span: ErrSpan,
    message: String,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct ErrSpan {
    pub fragment: String,
    pub line: u32,
    pub offset: usize,
}


impl std::error::Error for ParseError {}

impl ParseError {
    pub fn new(message: &str, span: Span) -> Self {
        Self { span: ParseError::to_err_span(span), message: message.to_string() }
    }

    fn to_err_span(span: Span) -> ErrSpan {
        ErrSpan {
            fragment: span.fragment().to_string(),
            line: span.location_line(),
            offset: span.location_offset(),
        }
    }

    pub fn update_message(&mut self, message: String) {
        self.message = message.to_string();
    }

    pub fn prepend_message(&mut self, message: String) {
        self.message = format!("{}: {}", message, self.message);
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ParseError: {}", self.message)
    }
}


// That's what makes it nom-compatible.
impl<'a> nom::error::ParseError<Span<'a>> for ParseError {
    fn from_error_kind(input: Span<'a>, kind: nom::error::ErrorKind) -> Self {
        Self::new(&format!("parse error {:?}", kind), input)
    }

    fn append(_input: Span<'a>, _kind: nom::error::ErrorKind, other: Self) -> Self {
        other
    }

    fn from_char(input: Span<'a>, c: char) -> Self {
        Self::new(&format!("unexpected character '{}'", c), input)
    }
}

impl<'a> nom::error::ContextError<Span<'a>> for ParseError {
    fn add_context(input: Span<'a>, ctx: &'static str, other: Self) -> Self {
        Self::new(&format!("{}: {}", ctx, other.message), input)
    }
}
