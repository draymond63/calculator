use crate::types::{Context, CResult, LatexExpr, Expr::{self, *}};
use crate::units::UnitVal;
use crate::error::Error;

use itertools::Itertools;


pub(crate) fn eval_mut_context(expr: &Expr, mut context: &mut Context) -> CResult<Option<UnitVal>> {
    eval_mut_context_def(expr, &mut context, None)
}

fn eval_mut_context_def(expr: &Expr, context: &mut Context, defining: Option<&str>) -> CResult<Option<UnitVal>> {
    match expr {
        EDefVar(var, expr) => {
            let result = eval_expr(expr, &context, Some(&var))?;
            if defining.is_some() {
                return Err(Error::EvalError(format!("Cannot contain nested variable definitions (variable '{}' & '{}')", var, defining.unwrap())));
            }
            if context.vars.contains_key(var) {
                return Err(Error::EvalError(format!("Variable '{var}' already defined")));
            }
            context.vars.insert(var.clone(), result.clone());
            Ok(Some(result))
        },
        EDefFunc(name, params, expr) => {
            if defining.is_some() {
                return Err(Error::EvalError(format!("Cannot contain nested variable definitions (variable '{}' & '{}')", name, defining.unwrap())));
            }
            if context.funcs.contains_key(name) {
                return Err(Error::EvalError(format!("Variable '{name}' already defined")));
            }
            context.funcs.insert(name.clone(), (params.clone(), *expr.clone()));
            Ok(None)
        },
        _ => Ok(Some(eval_expr(expr, context, defining)?)),
    }
}

fn compose_expr<F>(expr1: &Expr, expr2: &Expr, func: F, context: &Context, defining: Option<&str>) -> CResult<UnitVal>
    where F: Fn(UnitVal, UnitVal) -> UnitVal
{
    Ok(func(eval_expr(expr1, context, defining)?, eval_expr(expr2, context, defining)?))
}

fn eval_expr(expr: &Expr, context: &Context, defining: Option<&str>) -> CResult<UnitVal> {
    let compose = |expr1: &Expr, expr2: &Expr, func: fn(UnitVal, UnitVal) -> UnitVal| {
        compose_expr(expr1, expr2, func, context, defining)
    };

    match expr {
        ENum(num) => Ok(num.clone()),
        EAdd(expr1, expr2) => { eval_expr(expr1, context, defining)? + eval_expr(expr2, context, defining)? },
        ESub(expr1, expr2) => { eval_expr(expr1, context, defining)? - eval_expr(expr2, context, defining)? },
        EMul(expr1, expr2) => compose(expr1, expr2, |a, b| a * b),
        EDiv(expr1, expr2) => compose(expr1, expr2, |a, b| a / b),
        EExp(expr1, expr2) => {
            eval_expr(expr1, context, defining)?.powf(eval_expr(expr2, context, defining)?)
        },
        EVar(var) => {
            if defining.is_some() && var == defining.unwrap() {
                return Err(Error::EvalError(format!("Variable '{var}' cannot be defined recursively")))
            } else if let Some(val) = context.vars.get(var) {
                Ok(val.clone())
            } else {
                Err(Error::EvalError(format!("Variable '{var}' not defined")))
            }
        },
        EFunc(name, inputs) => {
            let applied_defaulted_func = apply_default_function(name, inputs, context, defining);
            if applied_defaulted_func.is_ok() {
                applied_defaulted_func
            } else if defining.is_some() && name == defining.unwrap() {
                Err(Error::EvalError(format!("Function '{name}' cannot be defined recursively")))
            } else if let Some((params, func_def)) = context.funcs.get(name) {
                if params.len() != inputs.len() {
                    return Err(Error::EvalError(format!("Function '{}' expects {} arguments, but got {}", name, params.len(), inputs.len())));
                }
                let mut eval_context = context.clone();

                for (param, input) in params.iter().zip(inputs.iter()) {
                    eval_context.vars.insert(param.clone(), eval_expr(input, context, defining)?);
                }
                eval_expr(func_def, &eval_context, defining)
            } else {
                Err(Error::EvalError(format!("Function '{name}' not defined")))
            }
        },
        ETex(expr) => eval_latex(expr, &context, defining),
        _ => Err(Error::EvalError(format!("Unexpected expression '{expr:?}'. Did you mean to call `eval_mut_context_def`?")),)
    }
}

fn apply_default_function(name: &String, inputs: &Vec<Expr>, context: &Context, defining: Option<&str>) -> CResult<UnitVal> {
    if inputs.len() > 1 {
        return Err(Error::EvalError("Default functions only accept one argument".to_string()));
    }
    let input = eval_expr(inputs.get(0).unwrap(), context, defining)?;
    let callable = match name.as_str() {
        "sin" => Some(Box::new(|x: f32| x.sin()) as Box<dyn Fn(f32) -> f32>),
        "cos" => Some(Box::new(|x: f32| x.cos()) as Box<dyn Fn(f32) -> f32>),
        "tan" => Some(Box::new(|x: f32| x.tan()) as Box<dyn Fn(f32) -> f32>),
        _ => None,
    };
    if let Some(callable) = callable {
        Ok(UnitVal::scalar(callable(input.as_scalar()?)))
    } else {
        Err(Error::EvalError(format!("Function '{name}' not defined")))
    }
}

fn eval_latex(expr: &LatexExpr, context: &Context, defining: Option<&str>) -> CResult<UnitVal> {
    let compose = |expr1: &Expr, expr2: &Expr, func: fn(UnitVal, UnitVal) -> UnitVal| {
        compose_expr(expr1, expr2, func, context, defining)
    };

    match expr.name.as_str() {
        "frac" => {
            if expr.params.len() != 2 {
                return Err(Error::EvalError("frac expects 2 arguments".to_string()));
            }
            if expr.subscript.is_some() || expr.superscript.is_some() {
                return Err(Error::EvalError("frac does not support subscripts or superscripts".to_string()));
            }
            let (num, denom) = expr.params.clone().into_iter().collect_tuple().unwrap();
            compose(&num, &denom, |a, b| a / b)
        },
        "sqrt" => {
            if expr.params.len() != 1 {
                return Err(Error::EvalError("Square root expects 1 argument".to_string()));
            }
            if expr.subscript.is_some() || expr.superscript.is_some() {
                return Err(Error::EvalError("Square root does not support subscripts or superscripts".to_string()));
            }
            let val = expr.params.get(0).unwrap();
            Ok(eval_expr(val, context, defining)?.sqrt()?)
        },
        "sum" => {
            if expr.params.len() != 1 || expr.subscript.is_none() || expr.superscript.is_none() {
                return Err(Error::EvalError(format!("Summation expects a parameter, a subscript, and a superscript, received {:?}", expr)));
            }
            let param = expr.params.get(0).unwrap();
            let superscript = expr.superscript.as_ref().unwrap();
            let subscript = expr.subscript.as_ref().unwrap();
            let up = eval_expr(&superscript, context, defining)?;

            let mut sum_context = context.clone();
            let ub_var = match *subscript.clone() {
                EDefVar(name, _) => Some(name),
                _ => None,
            };
            let ub = eval_mut_context_def(&subscript, &mut sum_context, defining)?.unwrap();

            // Ensure up and ub are integers
            if up.fract()? != 0.0 || ub.fract()? != 0.0 {
                return Err(Error::EvalError("Summation bounds must be integers".to_string()));
            }
            let up = up.as_scalar()? as i32 + 1;
            let ub = ub.as_scalar()? as i32;

            let mut sum = 0.0;
            for i in ub..up {
                if ub_var.is_some() {
                    sum_context.vars.insert(ub_var.clone().unwrap(), UnitVal::scalar(i as f32));
                }
                sum += eval_expr(&param, &sum_context, defining)?.as_scalar()?;
            }
            Ok(UnitVal::scalar(sum))
        },
        unknown_name => Err(Error::EvalError(format!("Unrecognized latex expression '{unknown_name}'")))
    }
}


#[cfg(test)]
pub(crate) fn evaluate(expr: Expr) -> UnitVal {
    let mut context = Context::new();
    eval_mut_context(&expr, &mut context).unwrap().unwrap()
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
    fn evaluate_enum_test() {
        let expr = num(1234.0);
        assert_eq!(evaluate(expr).as_scalar().unwrap(), 1234.0);
    }

    #[test]
    fn evaluate_eadd_test() {
        let expr = EAdd(boxed_num(12.0), boxed_num(34.0));
        assert_eq!(evaluate(expr).as_scalar().unwrap(), 46.0);
    }

    #[test]
    fn evaluate_easub_test() {
        let expr = ESub(boxed_num(12.0), boxed_num(34.0));
        assert_eq!(evaluate(expr).as_scalar().unwrap(), -22.0);
    }

    #[test]
    fn test_evaluate_nested_arithmetic_expression() {
        let expr = EAdd(
            Box::new(EMul(boxed_num(1.0), boxed_num(2.0))),
            Box::new(EDiv(
                Box::new(EExp(boxed_num(6.0), boxed_num(2.0))),
                boxed_num(5.0),
            )),
        );
        assert_eq!(evaluate(expr).as_scalar().unwrap(), 9.2);
    }

    #[test]
    fn test_variable_definition() {
        let expr = EDefVar(
            "a".to_string(),
            boxed_num(2.0),
        );
        let mut context = Context::new();
        assert_eq!(context.vars.get("a"), None);
        eval_mut_context_def(&expr, &mut context, None).unwrap();
        assert_eq!(context.vars.get("a"), Some(&UnitVal::scalar(2.0)));
    }

    #[test]
    fn test_function_definition_and_call() {
        // f(x,y) = x + y
        let expr = EDefFunc(
            "f".to_string(),
            vec!["x".to_string(), "y".to_string()],
            Box::new(EAdd(Box::new(EVar("x".to_string())), Box::new(EVar("y".to_string())))),
        );
        let mut context = Context::new();
        assert_eq!(context.funcs.get("f"), None);
        eval_mut_context_def(&expr, &mut context, None).unwrap();
        assert_ne!(context.funcs.get("f"), None);

        let call = EFunc("f".to_string(), vec![num(1.0), num(2.0)]);
        let unit_val = eval_mut_context_def(&call, &mut context, None).unwrap().unwrap();
        assert_eq!(unit_val.as_scalar().unwrap(), 3.0);
    }

    #[test]
    fn test_units_good() {
        let expr = EAdd(
            Box::new(EMul(boxed_num(1.0), Box::new(ENum(UnitVal::new_identity("km"))))),
            Box::new(EMul(boxed_num(1000.0), Box::new(ENum(UnitVal::new_identity("m"))))),
        );
        assert_eq!(evaluate(expr), UnitVal::new_value(2.0, "km"));
    }
}
