use std::collections::HashMap;
use nom;
use nom_locate::LocatedSpan;

use crate::unit_value::UnitVal;
use crate::error::{Error, ParseError};


pub type CResult<T> = Result<T, Error>;


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

    pub fn set_script_param(&mut self, script: char, expr: Expr) -> Result<(), &str> {
        match script {
            '^' => self.superscript = {
                if self.superscript.is_some() {
                    return Err("Superscript already set");
                }
                Some(Box::new(expr))
            },
            '_' => self.subscript = {
                if self.subscript.is_some() {
                    return Err("Subscript already set");
                }
                Some(Box::new(expr))
            },
            _ => return Err("Unknown script type")
        };
        Ok(())
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
pub type BaseParseResult<'a, T> = nom::IResult<Span<'a>, T, ParseError>;
pub type ParseResult<'a> = BaseParseResult<'a, Expr>;
pub type ParseResultStr<'a> = BaseParseResult<'a, Span<'a>>;
pub type ParseResultVec<'a> = BaseParseResult<'a, Vec<Expr>>;
