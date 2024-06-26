use std::collections::HashMap;
use nom;
use nom_locate::LocatedSpan;

use crate::units::UnitVal;


#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    ENum(UnitVal),
    EVar(String),
    EFunc(String, Vec<Expr>),
    EAdd(Box<Expr>, Box<Expr>),
    ESub(Box<Expr>, Box<Expr>),
    EMul(Box<Expr>, Box<Expr>),
    EDiv(Box<Expr>, Box<Expr>),
    EExp(Box<Expr>, Box<Expr>),
    ETex(LatexExpr),
    EDefVar(String, Box<Expr>),
    EDefFunc(String, Vec<String>, Box<Expr>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct LatexExpr {
    pub name: String,
    pub superscript: Option<Box<Expr>>,
    pub subscript: Option<Box<Expr>>,
    pub params: Vec<Expr>,
}

impl LatexExpr {
    pub fn new(name: String) -> Self {
        LatexExpr {
            name,
            superscript: None,
            subscript: None,
            params: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Context {
    pub vars: HashMap<String, UnitVal>,
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
pub type BaseParseResult<'a, T> = nom::IResult<Span<'a>, T, ParseError<'a>>;
pub type ParseResult<'a> = BaseParseResult<'a, Expr>;
pub type ParseResultStr<'a> = BaseParseResult<'a, Span<'a>>;
pub type ParseResultVec<'a> = BaseParseResult<'a, Vec<Expr>>;



#[derive(Debug, PartialEq)]
pub struct ParseError<'a> {
    span: Span<'a>,
    message: String,
}

impl<'a> ParseError<'a> {
    pub fn new(message: &str, span: Span<'a>) -> Self {
        Self { span, message: message.to_string() }
    }

    pub fn update_message(&mut self, message: &str) {
        self.message = message.to_string();
    }

    pub fn prepend_message(&mut self, message: &str) {
        self.message = format!("{}: {}", message, self.message);
    }
    // pub fn span(&self) -> &Span { &self.span }
    // pub fn line(&self) -> u32 { self.span().location_line() }
    // pub fn offset(&self) -> usize { self.span().location_offset() }
}

// That's what makes it nom-compatible.
impl<'a> nom::error::ParseError<Span<'a>> for ParseError<'a> {
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

impl<'a> nom::error::ContextError<Span<'a>> for ParseError<'a> {
    fn add_context(input: Span<'a>, ctx: &'static str, other: Self) -> Self {
        Self::new(&format!("{}: {}", ctx, other.message), input)
    }
}
