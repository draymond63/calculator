use crate::types::{
    Expr,
    Expr::*,
    ParseResult,
    ParseResultVec,
    ParseError,
    Span,
};

use crate::parsing_helpers::{start_alpha, cut_with_message, trim};

use nom::branch::alt;
use nom::character::complete::{char, digit1, space0};
use nom::bytes::complete::take_until;
use nom::combinator::{map, cut};
use nom::multi::{many0, separated_list0};
use nom::sequence::{delimited, tuple, pair};

use std::str::FromStr;



pub(crate) fn parse(input: Span) -> ParseResult {
    let (input, expr) = cut(parse_math_expr_or_def)(input)?;
    if !input.is_empty() {
        return Err(nom::Err::Failure(ParseError::new("Unexpected input", input)));
    }
    Ok(("".into(), expr))
}

fn parse_math_expr_or_def(input: Span) -> ParseResult {
    let (input, expr) = alt((parse_def, parse_math_expr))(input)?;
    Ok((input, expr))
}

fn parse_def(input: Span) -> ParseResult {
    let (rhs, lhs) = take_until("=")(input)?;
    let (lhs, var) = cut_with_message(
        trim(start_alpha)(lhs), 
        "Variable name must start with an alphabetic character",
    )?;
    let (rhs, _) = char('=')(rhs)?;
    let (rhs, _) = space0(rhs)?;
    let (rhs, expr) = cut(parse_math_expr)(rhs)?;
    if lhs.contains('(') {
        let (_, params) = cut_with_message(
            parse_call_params(lhs),
            "Function parameters must be enclosed in parentheses",
        )?;
        // Assert each params is just a Var and get the string that makes it
        let params = params.into_iter().map(|expr| match expr {
            EVar(var) => var,
            _ => panic!("Unexpected expression in function definition"),
        }).collect();
        Ok((rhs, EDefFunc(var.to_string(), params, Box::new(expr))))
    } else {
        Ok((rhs, EDefVar(var.to_string(), Box::new(expr))))
    }
}

fn parse_math_expr(input: Span) -> ParseResult {
    let (input, num1) = parse_term(input)?;
    let term_splitters = alt((char('+'), char('-'))); 
    let (input, exprs) = many0(tuple((term_splitters, parse_term)))(input)?;
    Ok((input, map_ops(num1, exprs)))
}

fn parse_term(input: Span) -> ParseResult {
    let (input, num1) = parse_factor(input)?;
    let term_splitters = alt((char('/'), char('*'))); 
    let (input, exprs) = many0(tuple((term_splitters, parse_factor)))(input)?;
    Ok((input, map_ops(num1, exprs)))
}

fn parse_factor(input: Span) -> ParseResult {
    let (input, num1) = parse_insides(input)?;
    let (input, exprs) = many0(tuple((char('^'), parse_factor)))(input)?;
    Ok((input, map_ops(num1, exprs)))
}

fn parse_insides(input: Span) -> ParseResult {
    alt((parse_parens, parse_implicit_multiply, parse_func_call, parse_number, parse_var_use))(input)
}

fn parse_implicit_multiply(input: Span) -> ParseResult {
    let (input, (num, var)) = pair(parse_number, alt((parse_parens, parse_var_use)))(input)?;
    Ok((input, EMul(Box::new(num), Box::new(var))))
}

fn parse_func_call(input: Span) -> ParseResult {
    println!("Parsing: {:?}", input.fragment());
    let (input, name) = start_alpha(input)?;
    println!("Name: {:?}", name.fragment());
    let (input, params) = parse_call_params(input)?;
    println!("Params: {:?}", params);
    Ok((input, EFunc(name.to_string(), params)))
}

fn parse_call_params(input: Span) -> ParseResultVec {
    let (rest, param_insides) = delimited(char('('), take_until(")"), char(')'))(input)?;
    let (is_empty, params) = separated_list0(char(','), parse_math_expr)(param_insides)?;
    if !is_empty.is_empty() {
        return Err(nom::Err::Failure(ParseError::new("Call param input remain unparsed", is_empty)));
    }
    Ok((rest, params))
}

fn parse_number(input: Span) -> ParseResult {
    map(trim(digit1), parse_enum)(input)
}

fn parse_enum(parsed_num: Span) -> Expr {
    let num = f32::from_str(parsed_num.fragment()).unwrap();
    ENum(num)
}

fn parse_var_use(input: Span) -> ParseResult {
    map(trim(start_alpha), parse_evar)(input)
}

fn parse_evar(input: Span) -> Expr {
    match *input.fragment() {
        "e" => ENum(std::f32::consts::E),
        "pi" => ENum(std::f32::consts::PI),
        _ => EVar(input.to_string()),
    }
}

fn parse_parens(input: Span) -> ParseResult {
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
        let (_, parsed) = parse("12 + 34".into()).unwrap();
        let expected = EAdd(Box::new(ENum(12.0)), Box::new(ENum(34.0)));
        assert_eq!(parsed, expected);
    }

    #[test]
    fn parse_subtract_statement() {
        let (_, parsed) = parse("12 - 34".into()).unwrap();
        let expected = ESub(Box::new(ENum(12.0)), Box::new(ENum(34.0)));
        assert_eq!(parsed, expected);
    }

    #[test]
    fn parse_nested_add_sub_statements() {
        let (_, parsed) = parse("12 - 34 + 15 - 9".into()).unwrap();
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
        let (_, parsed) = parse("1 * 2 + 3 / 4 ^ 6".into()).unwrap();
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
        let (_, parsed) = parse("(1 + 2) * 3".into()).unwrap();
        let expected = EMul(
            Box::new(EAdd(Box::new(ENum(1.0)), Box::new(ENum(2.0)))),
            Box::new(ENum(3.0)),
        );
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_variable_definition() {
        let (_, parsed) = parse("a = 2".into()).unwrap();
        let expected = EDefVar(
            "a".to_string(),
            Box::new(ENum(2.0)),
        );
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_function_definition() {
        let (_, parsed) = parse("f(x, y) = x + y".into()).unwrap();
        let expected = EDefFunc(
            "f".to_string(),
            vec!["x".to_string(), "y".to_string()],
            Box::new(EAdd(Box::new(EVar("x".to_string())), Box::new(EVar("y".to_string())))),
        );
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_function_call() {
        let (_, parsed) = parse("f(1,a)".into()).unwrap();
        println!("{:?}", parsed);
        let expected = EFunc(
            "f".to_string(),
            vec![ENum(1.0), EVar("a".to_string())],
        );
        assert_eq!(parsed, expected);
    }
}
