use std::collections::HashMap;
use nom;
use nom_locate::LocatedSpan;


#[derive(Debug, PartialEq)]
pub enum Expr {
    ENum(f32),
    EVar(String),
    // EFunc(String, Vec<Expr>),
    EAdd(Box<Expr>, Box<Expr>),
    ESub(Box<Expr>, Box<Expr>),
    EMul(Box<Expr>, Box<Expr>),
    EDiv(Box<Expr>, Box<Expr>),
    EExp(Box<Expr>, Box<Expr>),
    EDefVar(String, Box<Expr>),
    // EDefFunc(String, Vec<String>, Box<Expr>),
}

#[derive(Debug)]
pub struct Context {
    pub vars: HashMap<String, f32>,
    pub funcs: HashMap<String, (Vec<String>, Expr)>,
}

impl Context {
    pub fn new() -> Self {
        Context {
            vars: HashMap::new(),
            funcs: HashMap::new(),
        }
    }
}


pub type Span<'a> = LocatedSpan<&'a str>;
pub type ParseResult<'a> = nom::IResult<Span<'a>, Expr, ParseError<'a>>;
pub type ParseResultStr<'a> = nom::IResult<Span<'a>, Span<'a>, ParseError<'a>>;



#[derive(Debug, PartialEq)]
pub struct ParseError<'a> {
    span: Span<'a>,
    message: String,
}

impl<'a> ParseError<'a> {
    pub fn new(message: String, span: Span<'a>) -> Self {
        Self { span, message }
    }

    // pub fn span(&self) -> &Span { &self.span }
    // pub fn line(&self) -> u32 { self.span().location_line() }
    // pub fn offset(&self) -> usize { self.span().location_offset() }
}

// That's what makes it nom-compatible.
impl<'a> nom::error::ParseError<Span<'a>> for ParseError<'a> {
    fn from_error_kind(input: Span<'a>, kind: nom::error::ErrorKind) -> Self {
        Self::new(format!("parse error {:?}", kind), input)
    }

    fn append(_input: Span<'a>, _kind: nom::error::ErrorKind, other: Self) -> Self {
        other
    }

    fn from_char(input: Span<'a>, c: char) -> Self {
        Self::new(format!("unexpected character '{}'", c), input)
    }
}

impl<'a> nom::error::ContextError<Span<'a>> for ParseError<'a> {
    fn add_context(input: Span<'a>, ctx: &'static str, other: Self) -> Self {
        Self::new(format!("{}: {}", ctx, other.message), input)
    }
}
