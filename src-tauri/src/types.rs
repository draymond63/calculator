use std::collections::HashMap;
use nom;
use nom_locate::LocatedSpan;

use crate::error::{Error, ParseError};


pub type CResult<T> = Result<T, Error>;


pub trait BaseField<'a>: 
    std::fmt::Debug + Clone +
    std::fmt::Display +
    serde::Serialize +
    std::convert::TryFrom<&'a str, Error=Box<dyn std::error::Error>> +
    std::convert::From<f64> +
    std::ops::Add<Output = CResult<Self>> +
    std::ops::Sub<Output = CResult<Self>> +
    std::ops::Mul<Output = Self> +
    std::ops::Div<Output = Self>
{
    fn as_scalar(&self) -> CResult<f64>;
    fn powf(&self, exp: Self) -> CResult<Self>;
    fn root(&self, n: Self) -> CResult<Self>;
    fn fract(&self) -> CResult<f64>;
    fn sin(&self) -> CResult<Self>;
    fn cos(&self) -> CResult<Self>;
    fn tan(&self) -> CResult<Self>;
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr<T> where for<'a> T: BaseField<'a> {
    ENum(T),
    EVar(String),
    EFunc(String, Vec<Expr<T>>),
    EAdd(Box<Expr<T>>, Box<Expr<T>>),
    ESub(Box<Expr<T>>, Box<Expr<T>>),
    EMul(Box<Expr<T>>, Box<Expr<T>>),
    EDiv(Box<Expr<T>>, Box<Expr<T>>),
    EExp(Box<Expr<T>>, Box<Expr<T>>),
    ETex(LatexExpr<T>),
    EDefVar(String, Box<Expr<T>>),
    EDefFunc(String, Vec<String>, Box<Expr<T>>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct LatexExpr<T> where for<'a> T: BaseField<'a> {
    pub name: String,
    pub superscript: Option<Box<Expr<T>>>,
    pub subscript: Option<Box<Expr<T>>>,
    pub params: Vec<Expr<T>>,
}

impl<T> LatexExpr<T> where for<'a> T: BaseField<'a> {
    pub fn set_script_param(&mut self, script: char, expr: Expr<T>) -> Result<(), &str> {
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
pub struct Context<T> where for<'a> T: BaseField<'a> {
    pub vars: HashMap<String, T>,
    pub funcs: HashMap<String, (Vec<String>, Expr<T>)>,
}

impl<T> Context<T> where for<'a> T: BaseField<'a> {
    pub fn new() -> Self {
        Context {
            vars: HashMap::new(),
            funcs: HashMap::new(),
        }
    }
}


pub type Span<'a> = LocatedSpan<&'a str>;
pub type BaseParseResult<'a, T> = nom::IResult<Span<'a>, T, ParseError>;
pub type ParseResult<'a, T> = BaseParseResult<'a, Expr<T>>;
pub type ParseResultStr<'a> = BaseParseResult<'a, Span<'a>>;
pub type ParseResultVec<'a, T> = BaseParseResult<'a, Vec<Expr<T>>>;
