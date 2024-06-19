use crate::types::Expr;
use crate::types::Expr::*;

use nom::branch::alt;
use nom::character::complete::{char, digit1, space0, alphanumeric1};
use nom::combinator::map;
use nom::multi::many0;
use nom::sequence::{delimited, tuple};
use nom::IResult;

use std::str::FromStr;
use std::error::Error;


pub(crate) fn parse(input: &str) -> Result<Expr, Box<dyn Error>> {
    let iresult = parse_math_expr_or_def(input);
    if iresult.is_ok() {
        let (input, result) = iresult.unwrap();
        if !input.is_empty() {
            panic!("parsing error, input remaining {:?}", input);
        }
        Ok(result)
    } else {
        panic!("{}", iresult.unwrap_err());
    }
}

fn parse_math_expr_or_def(input: &str) -> IResult<&str, Expr> {
    let (input, expr) = alt((parse_def, parse_math_expr))(input)?;
    Ok((input, expr))
}

fn parse_def(input: &str) -> IResult<&str, Expr> {
    let (input, _) = space0(input)?;
    let (input, var) = alphanumeric1(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char('=')(input)?;
    let (input, _) = space0(input)?;
    let (input, expr) = parse_math_expr(input)?;
    Ok((input, EDefVar(var.to_string(), Box::new(expr))))
}

fn parse_math_expr(input: &str) -> IResult<&str, Expr> {
    let (input, num1) = parse_term(input)?;
    let (input, exprs) = many0(tuple((alt((char('+'), char('-'))), parse_term)))(input)?;
    Ok((input, map_ops(num1, exprs)))
}

fn parse_term(input: &str) -> IResult<&str, Expr> {
    let (input, num1) = parse_factor(input)?;
    let (input, exprs) = many0(tuple((alt((char('/'), char('*'))), parse_factor)))(input)?;
    Ok((input, map_ops(num1, exprs)))
}

fn parse_factor(input: &str) -> IResult<&str, Expr> {
    let (input, num1) = parse_insides(input)?;
    let (input, exprs) = many0(tuple((char('^'), parse_factor)))(input)?;
    Ok((input, map_ops(num1, exprs)))
}

fn parse_insides(input: &str) -> IResult<&str, Expr> {
    alt((parse_parens, parse_number, parse_var_use))(input)
}

fn parse_number(input: &str) -> IResult<&str, Expr> {
    map(delimited(space0, digit1, space0), parse_enum)(input)
}

fn parse_enum(parsed_num: &str) -> Expr {
    let num = f32::from_str(parsed_num).unwrap();
    ENum(num)
}

fn parse_var_use(input: &str) -> IResult<&str, Expr> {
    map(delimited(space0, alphanumeric1, space0), parse_evar)(input)
}

fn parse_evar(input: &str) -> Expr {
    EVar(input.to_string())
}

fn parse_parens(input: &str) -> IResult<&str, Expr> {
    delimited(
        space0,
        delimited(char('('), parse_math_expr, char(')')), // This is the recursive call
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
        let parsed = parse("12 + 34").unwrap();
        let expected = EAdd(Box::new(ENum(12.0)), Box::new(ENum(34.0)));
        assert_eq!(parsed, expected);
    }

    #[test]
    fn parse_subtract_statement() {
        let parsed = parse("12 - 34").unwrap();
        let expected = ESub(Box::new(ENum(12.0)), Box::new(ENum(34.0)));
        assert_eq!(parsed, expected);
    }

    #[test]
    fn parse_nested_add_sub_statements() {
        let parsed = parse("12 - 34 + 15 - 9").unwrap();
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
        let parsed = parse("1 * 2 + 3 / 4 ^ 6").unwrap();
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
        let parsed = parse("(1 + 2) * 3").unwrap();
        let expected = EMul(
            Box::new(EAdd(Box::new(ENum(1.0)), Box::new(ENum(2.0)))),
            Box::new(ENum(3.0)),
        );
        assert_eq!(parsed, expected);
    }
}
