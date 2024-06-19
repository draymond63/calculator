use std::collections::HashMap;


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
