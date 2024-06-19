use crate::types::Expr;
use crate::types::Expr::*;

use nom::branch::alt;
use nom::character::complete::{char, digit1, space0, alphanumeric1};
use nom::combinator::map;
use nom::multi::many0;
use nom::sequence::{delimited, tuple};
use nom::IResult;
use nom::error::{context, VerboseError};

use std::str::FromStr;


pub type ParseResult<'a> = IResult<&'a str, Expr, VerboseError<&'a str>>; 

pub(crate) fn parse<'a>(input: &'a str) -> ParseResult<'a> {
    let (input, expr) = raise_err_to_failure(parse_math_expr_or_def(input))?;
    if !input.is_empty() {
        panic!("Parser returned Ok, but there is still input left: {:?}", input);
    }
    Ok(("", expr))
}

fn parse_math_expr_or_def<'a>(input: &'a str) -> ParseResult<'a> {
    let (input, expr) = alt((context("definition line", parse_def), context("expression line", parse_math_expr)))(input)?;
    Ok((input, expr))
}

fn parse_def<'a>(input: &'a str) -> ParseResult<'a> {
    let (input, var) = delimited(space0, alphanumeric1, space0)(input)?;
    let (input, _) = char('=')(input)?;
    let (input, _) = space0(input)?;
    let (input, expr) = raise_err_to_failure(parse_math_expr(input))?;
    Ok((input, EDefVar(var.to_string(), Box::new(expr))))
}

fn raise_err_to_failure<'a>(result: ParseResult<'a>) -> ParseResult<'a> {
    match result {
        Ok((input, result)) => Ok((input, result)),
        Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
            return Err(nom::Err::Failure(e));
        }
        Err(e) => panic!("Unexpected parse error: {:?}", e),
    }
}

fn parse_math_expr<'a>(input: &'a str) -> ParseResult<'a> {
    let (input, num1) = parse_term(input)?;
    let term_splitters = alt((context("add", char('+')), context("subtract", char('-')))); 
    let (input, exprs) = many0(tuple((term_splitters, parse_term)))(input)?;
    Ok((input, map_ops(num1, exprs)))
}

fn parse_term<'a>(input: &'a str) -> ParseResult<'a> {
    let (input, num1) = parse_factor(input)?;
    let term_splitters = alt((context("divide", char('/')), context("multiply", char('*')))); 
    let (input, exprs) = many0(tuple((term_splitters, context("exponent", parse_factor))))(input)?;
    Ok((input, map_ops(num1, exprs)))
}

fn parse_factor<'a>(input: &'a str) -> ParseResult<'a> {
    let (input, num1) = parse_insides(input)?;
    let (input, exprs) = many0(tuple((char('^'), context("exponent", parse_factor))))(input)?;
    Ok((input, map_ops(num1, exprs)))
}

fn parse_insides<'a>(input: &'a str) -> ParseResult<'a> {
    alt((parse_parens, parse_number, parse_var_use))(input)
}

fn parse_number<'a>(input: &'a str) -> ParseResult<'a> {
    map(delimited(space0, context("number", digit1), space0), parse_enum)(input)
}

fn parse_enum(parsed_num: &str) -> Expr {
    let num = f32::from_str(parsed_num).unwrap();
    ENum(num)
}

fn parse_var_use<'a>(input: &'a str) -> ParseResult<'a> {
    map(delimited(space0, context("variable", alphanumeric1), space0), parse_evar)(input)
}

fn parse_evar(input: &str) -> Expr {
    EVar(input.to_string())
}

fn parse_parens<'a>(input: &'a str) -> ParseResult<'a> {
    delimited(
        space0,
        delimited(char('('), context("brackets", parse_math_expr), char(')')), // This is the recursive call
        space0,
    )(input)
}

fn map_ops(expr: Expr, rem: Vec<(char, Expr)>) -> Expr {
    rem.into_iter().fold(expr, |acc, val| parse_op(val, acc))
}

fn parse_op(tup: (char, Expr), expr1: Expr) -> Expr {
    let (op, expr2) = tup;
    match op {
        '+' => EAdd(Box::new(expr1), Box::new(expr2)),
        '-' => ESub(Box::new(expr1), Box::new(expr2)),
        '*' => EMul(Box::new(expr1), Box::new(expr2)),
        '/' => EDiv(Box::new(expr1), Box::new(expr2)),
        '^' => EExp(Box::new(expr1), Box::new(expr2)),
        _ => panic!("Unknown Operation, {:?}", op),
    }
}


#[cfg(test)]
mod tests {
    use crate::parser::parse;
    use crate::types::Expr::*;

    #[test]
    fn parse_add_statement() {
        let (_, parsed) = parse("12 + 34").unwrap();
        let expected = EAdd(Box::new(ENum(12.0)), Box::new(ENum(34.0)));
        assert_eq!(parsed, expected);
    }

    #[test]
    fn parse_subtract_statement() {
        let (_, parsed) = parse("12 - 34").unwrap();
        let expected = ESub(Box::new(ENum(12.0)), Box::new(ENum(34.0)));
        assert_eq!(parsed, expected);
    }

    #[test]
    fn parse_nested_add_sub_statements() {
        let (_, parsed) = parse("12 - 34 + 15 - 9").unwrap();
        let expected = ESub(
            Box::new(EAdd(
                Box::new(ESub(Box::new(ENum(12.0)), Box::new(ENum(34.0)))),
                Box::new(ENum(15.0))
            )),
            Box::new(ENum(9.0))
        );
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_parse_multi_level_expression() {
        let (_, parsed) = parse("1 * 2 + 3 / 4 ^ 6").unwrap();
        let expected = EAdd(
            Box::new(EMul(Box::new(ENum(1.0)), Box::new(ENum(2.0)))),
            Box::new(EDiv(
                Box::new(ENum(3.0)),
                Box::new(EExp(Box::new(ENum(4.0)), Box::new(ENum(6.0)))),
            )),
        );
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_parse_expression_with_parantheses() {
        let (_, parsed) = parse("(1 + 2) * 3").unwrap();
        let expected = EMul(
            Box::new(EAdd(Box::new(ENum(1.0)), Box::new(ENum(2.0)))),
            Box::new(ENum(3.0)),
        );
        assert_eq!(parsed, expected);
    }
}
