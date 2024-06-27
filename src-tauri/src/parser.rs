use crate::types::{*, Expr::*};
use crate::parsing_helpers::*;
use crate::units::UnitVal;

use nom::branch::alt;
use nom::character::complete::{char, digit1, space0};
use nom::bytes::complete::take_until;
use nom::combinator::map;
use nom::multi::{many0, separated_list0};
use nom::sequence::{delimited, tuple, pair};

use std::str::FromStr;



pub(crate) fn parse(input: Span) -> ParseResult {
    let (input, expr) = parse_math_expr_or_def(input)?;
    if !input.is_empty() {
        return Err(nom::Err::Failure(ParseError::new("Terminated parsing early", input)));
    }
    Ok(("".into(), expr))
}

fn parse_math_expr_or_def(input: Span) -> ParseResult {
    let (input, expr) = alt((parse_def, parse_math_expr))(input)?;
    Ok((input, expr))
}

fn parse_def(input: Span) -> ParseResult {
    let (rhs, lhs) = take_until("=")(input)?;
    if lhs.contains('{') {
        return Err(nom::Err::Error(
            ParseError::new("Matched definition, but it's most likely within a latex command. Is this corect?", lhs)
        ));
    }
    let (lhs, var) = mcut(trim(start_alpha), "Variable name must start with an alphabetic character")(lhs)?;
    let (rhs, _) = char('=')(rhs)?;
    let (rhs, _) = space0(rhs)?;
    let (rhs, expr) = prepend_cut(parse_math_expr, "In RHS of definition")(rhs)?;
    if lhs.contains('(') {
        let (_, params) = mcut(parse_call_params,"Invalid function parameters")(lhs)?;
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
    // println!("expr -> term: {:?}", input.fragment());
    let (input, num1) = parse_term(input)?;
    let term_splitters = alt((char('+'), char('-'))); 
    // println!("expr -> term2: {:?}", input.fragment());
    let (input, exprs) = many0(tuple((term_splitters, parse_term)))(input)?;
    // println!("expr done");
    Ok((input, map_ops(num1, exprs)))
}

fn parse_term(input: Span) -> ParseResult {
    // println!("term -> factor: {:?}", input.fragment());
    let (input, num1) = parse_term_no_fractions(input)?;
    let term_splitters = alt((char('/'), char('*'))); 
    // println!("term -> factor2: {:?}", input.fragment());
    let (input, exprs) = many0(tuple((term_splitters, parse_term_no_fractions)))(input)?;
    // println!("term done");
    Ok((input, map_ops(num1, exprs)))
}

fn parse_term_no_fractions(input: Span) -> ParseResult {
    // println!("factor -> insides: {:?}", input.fragment());
    let (input, base) = parse_component(input)?;
    // println!("factor -> factor: {:?}", input.fragment());
    let (input, exprs) = many0(tuple((char('^'), parse_term_no_fractions)))(input)?;
    // println!("factor done");
    Ok((input, map_ops(base, exprs)))
}

fn parse_component(input: Span) -> ParseResult {
    // println!("insides -> alt: {:?}", input.fragment());
    let (input, _) = space0(input)?;
    alt((parse_parens, parse_implicit_multiply, parse_func_call, parse_latex, parse_number, parse_var_use))(input)
}

fn parse_implicit_multiply(input: Span) -> ParseResult {
    let (input, (num, var)) = 
    pair(
        parse_number, 
        alt((parse_parens, parse_var_use, parse_func_call, parse_latex))
    )(input)?;
    // println!("found implicit multiply");
    Ok((input, EMul(Box::new(num), Box::new(var))))
}

fn parse_func_call(input: Span) -> ParseResult {
    let (input, name) = start_alpha(input)?;
    let (input, params) = parse_call_params(input)?;
    // println!("found func call");
    Ok((input, EFunc(name.to_string(), params)))
}

fn parse_call_params(input: Span) -> ParseResultVec {
    let (rest, param_insides) = alt((
        unwrap("(", ")"),
        unwrap("\\left(", "\\right)"),
    ))(input)?;
    let (is_empty, params) = separated_list0(char(','), parse_math_expr)(param_insides)?;
    if !is_empty.is_empty() {
        return Err(nom::Err::Failure(ParseError::new("Call param input remain unparsed", is_empty)));
    }
    Ok((rest, params))
}

fn parse_latex(input: Span) -> ParseResult {
    // println!("testing for latex: {:?}", input.fragment());
    let (rest, _) = char('\\')(input)?;
    // println!("found latex");
    let (rest, name) = mcut(start_alpha, "Latex command must be followed by a name")(rest)?;
    let mut latex_expr = LatexExpr::new(name.to_string());
    let mut remaining_input = rest;
    let mut found_params = false;

    // println!("latex -> super: {:?}", remaining_input.fragment());
    let superscript = parse_latex_param(remaining_input, '^', false);
    if superscript.is_ok() {
        (remaining_input, latex_expr.superscript) = superscript.unwrap();
        found_params = true;
    }
    // println!("latex -> sub: {:?}", remaining_input.fragment());
    let subscript = parse_latex_param(remaining_input, '_', true);
    if subscript.is_ok() {
        (remaining_input, latex_expr.subscript) = subscript.unwrap();
        found_params = true;
    }
    let mut params = unwrap("{", "}")(remaining_input);
    while params.is_ok() {
        let (rest, inside) = params.unwrap();
        // println!("latex param -> component: {:?}", inside.fragment());
        let (_, expr) = prepend_cut(parse_math_expr, "In latex param")(inside)?;
        latex_expr.params.push(expr);
        params = unwrap("{", "}")(rest);
        remaining_input = rest;
        found_params = true;
    }
    if !found_params {
        if let Ok(num) = match_const(name) {
            return Ok((remaining_input, num));
        } else {
            return Err(nom::Err::Failure(ParseError::new("Latex command must have at least one parameter", input)));
        }
    }
    Ok((remaining_input, ETex(latex_expr)))
}


fn parse_latex_param(input: Span, c: char, allow_def: bool) -> BaseParseResult<Option<Box<Expr>>> {
    let (input, _) = char(c)(input)?;
    let (rest, inside) = safe_unwrap("{", "}")(input);
    let (_, expr) = if allow_def {
        prepend_cut(parse_math_expr_or_def, "In latex param")(inside)?
    } else {
        prepend_cut(parse_math_expr, "In latex param")(inside)?
    };
    Ok((rest, Some(Box::new(expr))))
}

fn parse_number(input: Span) -> ParseResult {
    map(trim(digit1), parse_enum)(input)
}

fn parse_enum(parsed_num: Span) -> Expr {
    let num = f32::from_str(parsed_num.fragment()).unwrap();
    ENum(UnitVal::scalar(num))
}

fn parse_var_use(input: Span) -> ParseResult {
    map(trim(start_alpha), parse_evar)(input)
}

fn parse_evar(input: Span) -> Expr {
    match_const(input)
        .or_else(|_| match_unit(input))
        .or_else(|_| Ok::<Expr, &str>(EVar(input.fragment().to_string()))).unwrap()
}

fn match_unit(input: Span) -> Result<Expr, &str> {
    if UnitVal::is_valid_unit(input.fragment()) {
        Ok(ENum(UnitVal::new_identity(input.fragment())))
    } else {
        Err("Invalid unit")
    }
}

fn match_const(input: Span) -> Result<Expr, &str> {
    match *input.fragment() {
        "e" => Ok(ENum(UnitVal::scalar(std::f32::consts::E))),
        "pi" => Ok(ENum(UnitVal::scalar(std::f32::consts::PI))),
        _ => Err("Unknown constant"),
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
    use super::*;

    fn num(x: f32) -> Expr {
        ENum(UnitVal::scalar(x))
    }

    fn boxed_num(x: f32) -> Box<Expr> {
        Box::new(num(x))
    }

    #[test]
    fn parse_add_statement() {
        let (_, parsed) = parse("12 + 34".into()).unwrap();
        let expected = EAdd(boxed_num(12.0), boxed_num(34.0));
        assert_eq!(parsed, expected);
    }

    #[test]
    fn parse_subtract_statement() {
        let (_, parsed) = parse("12 - 34".into()).unwrap();
        let expected = ESub(boxed_num(12.0), boxed_num(34.0));
        assert_eq!(parsed, expected);
    }

    #[test]
    fn parse_nested_add_sub_statements() {
        let (_, parsed) = parse("12 - 34 + 15 - 9".into()).unwrap();
        let expected = ESub(
            Box::new(EAdd(
                Box::new(ESub(boxed_num(12.0), boxed_num(34.0))),
                boxed_num(15.0)
            )),
            boxed_num(9.0)
        );
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_parse_multi_level_expression() {
        let (_, parsed) = parse("1 * 2 + 3 / 4 ^ 6".into()).unwrap();
        let expected = EAdd(
            Box::new(EMul(boxed_num(1.0), boxed_num(2.0))),
            Box::new(EDiv(
                boxed_num(3.0),
                Box::new(EExp(boxed_num(4.0), boxed_num(6.0))),
            )),
        );
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_parse_expression_with_parantheses() {
        let (_, parsed) = parse("(1 + 2) * 3".into()).unwrap();
        let expected = EMul(
            Box::new(EAdd(boxed_num(1.0), boxed_num(2.0))),
            boxed_num(3.0),
        );
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_variable_definition() {
        let (_, parsed) = parse("a = 2".into()).unwrap();
        let expected = EDefVar(
            "a".to_string(),
            boxed_num(2.0),
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
        let expected = EFunc(
            "f".to_string(),
            vec![num(1.0), EVar("a".to_string())],
        );
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_latex() {
        let (_, parsed) = parse("\\frac{1}{2}".into()).unwrap();
        let expected = ETex(
            LatexExpr {
                name: "frac".to_string(),
                superscript: None,
                subscript: None,
                params: vec![num(1.0), num(2.0)],
            }
        );
        assert_eq!(parsed, expected);

        let (_, parsed) = parse("\\sum^{3}_{i=1}{i}".into()).unwrap();
        let expected = ETex(
            LatexExpr {
                name: "sum".to_string(),
                superscript: Some(Box::new(num(3.0))),
                subscript: Some(Box::new(EDefVar("i".to_string(), Box::new(num(1.0))))),
                params: vec![EVar("i".to_string())],
            }
        );
        assert_eq!(parsed, expected);
        // TODO: Allow superscript and subscript to be any order
        // let (_, parsed) = parse("\\sum_{i=1}^{3}{i}".into()).unwrap();
        // assert_eq!(parsed, expected);
    }

    #[test]
    fn test_units() {
        let (_, parsed) = parse("1 km + 1 m".into()).unwrap();
        let expected = EAdd(
            Box::new(EMul(boxed_num(1.0), Box::new(ENum(UnitVal::new_identity("km"))))),
            Box::new(EMul(boxed_num(1.0), Box::new(ENum(UnitVal::new_identity("m"))))),
        );
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_full() {
        let (_, parsed) = parse("f(x, y) = x + \\sum^{3}_{i=1}{i*y}".into()).unwrap();
        let expected = EDefFunc(
            "f".to_string(),
            vec!["x".to_string(), "y".to_string()],
            Box::new(EAdd(
                Box::new(EVar("x".to_string())),
                Box::new(ETex(
                    LatexExpr {
                        name: "sum".to_string(),
                        superscript: Some(boxed_num(3.0)),
                        subscript: Some(Box::new(EDefVar("i".to_string(), boxed_num(1.0)))),
                        params: vec![EMul(Box::new(EVar("i".to_string())), Box::new(EVar("y".to_string())))],
                    }
                ))
            ))
        );
        assert_eq!(parsed, expected);
    }
}
