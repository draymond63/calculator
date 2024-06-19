use crate::types::{
    Expr,
    Expr::*,
    ParseResult,
    ParseResultVec,
    ParseError,
    Span,
};

use crate::parsing_helpers::{start_alpha, cut_with_message};

use nom::branch::alt;
use nom::character::complete::{char, digit1, space0};
use nom::bytes::complete::take_until;
use nom::combinator::{map, cut};
use nom::multi::{many0, many0_count, separated_list1};
use nom::sequence::{delimited, tuple, pair};

use std::str::FromStr;



pub(crate) fn parse<'a>(input: Span<'a>) -> ParseResult<'a> {
    let (input, expr) = cut(parse_math_expr_or_def)(input)?;
    if !input.is_empty() {
        return Err(nom::Err::Failure(ParseError::new("Unexpected input", input)));
    }
    Ok(("".into(), expr))
}

fn parse_math_expr_or_def<'a>(input: Span<'a>) -> ParseResult<'a> {
    let (input, expr) = alt((parse_def, parse_math_expr))(input)?;
    Ok((input, expr))
}

fn parse_def<'a>(input: Span<'a>) -> ParseResult<'a> {
    let (input, name_side) = take_until("=")(input)?;
    let (name_side, var) = cut_with_message(
        delimited(space0, start_alpha, space0)(name_side), 
        "Variable name must start with an alphabetic character",
    )?;
    let (input, _) = char('=')(input)?;
    let (input, _) = space0(input)?;
    let (input, expr) = cut(parse_math_expr)(input)?;
    if name_side.contains('(') {
        let (_, param_inner) = cut_with_message(
            delimited(char('('), take_until(")"), char(')'))(name_side),
            "Function parameters must be enclosed in parentheses",
        )?;
        let (_, params) = separated_list1(char(','), cut(start_alpha))(param_inner)?;
        let params = params.into_iter().map(|s| s.fragment().to_string()).collect();
        Ok((input, EDefFunc(var.to_string(), params, Box::new(expr))))
    } else {
        Ok((input, EDefVar(var.to_string(), Box::new(expr))))
    }
}

fn parse_math_expr<'a>(input: Span<'a>) -> ParseResult<'a> {
    let (input, num1) = parse_term(input)?;
    let term_splitters = alt((char('+'), char('-'))); 
    let (input, exprs) = many0(tuple((term_splitters, parse_term)))(input)?;
    Ok((input, map_ops(num1, exprs)))
}

fn parse_term<'a>(input: Span<'a>) -> ParseResult<'a> {
    let (input, num1) = parse_factor(input)?;
    let term_splitters = alt((char('/'), char('*'))); 
    let (input, exprs) = many0(tuple((term_splitters, parse_factor)))(input)?;
    Ok((input, map_ops(num1, exprs)))
}

fn parse_factor<'a>(input: Span<'a>) -> ParseResult<'a> {
    let (input, num1) = parse_insides(input)?;
    let (input, exprs) = many0(tuple((char('^'), parse_factor)))(input)?;
    Ok((input, map_ops(num1, exprs)))
}

fn parse_insides<'a>(input: Span<'a>) -> ParseResult<'a> {
    alt((parse_parens, parse_implicit_multiply, parse_func_call, parse_number, parse_var_use))(input)
}

fn parse_implicit_multiply<'a>(input: Span<'a>) -> ParseResult<'a> {
    let (input, (num, var)) = pair(parse_number, alt((parse_parens, parse_var_use)))(input)?;
    Ok((input, EMul(Box::new(num), Box::new(var))))
}

fn parse_func_call<'a>(input: Span<'a>) -> ParseResult<'a> {
    let (input, name) = start_alpha(input)?;
    let (input, params) = parse_call_params(input)?;
    Ok((input, EFunc(name.to_string(), params)))
}

fn parse_call_params<'a>(input: Span<'a>) -> ParseResultVec<'a> {
    let (input, _) = char('(')(input)?;
    // TODO: Which "input" should be returned?
    let (_, param_insides) = take_until(")")(input)?;
    let (input, params) = separated_list1(char(','), parse_math_expr)(param_insides)?;
    Ok((input, params))
}

fn parse_number<'a>(input: Span<'a>) -> ParseResult<'a> {
    map(delimited(space0, digit1, space0), parse_enum)(input)
}

fn parse_enum(parsed_num: Span) -> Expr {
    let num = f32::from_str(parsed_num.fragment()).unwrap();
    ENum(num)
}

fn parse_var_use<'a>(input: Span<'a>) -> ParseResult<'a> {
    map(delimited(space0, start_alpha, space0), parse_evar)(input)
}

fn parse_evar<'a>(input: Span<'a>) -> Expr {
    match *input.fragment() {
        "e" => ENum(std::f32::consts::E),
        "pi" => ENum(std::f32::consts::PI),
        _ => EVar(input.to_string()),
    }
}

fn parse_parens<'a>(input: Span<'a>) -> ParseResult<'a> {
    let (_, open_count) = many0_count(char::<Span<'a>, ParseError<'a>>('('))(input)?;
    let (_, close_count) = many0_count(char::<Span<'a>, ParseError<'a>>(')'))(input)?;
    if open_count != close_count {
        return Err(nom::Err::Failure(ParseError::new(&format!("Mismatched parentheses ({open_count}v{close_count})"), input)));
    }
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
}
