use crate::types::{Context, CResult, LatexExpr, Expr::{self, *}};
use crate::unit_value::UnitVal;
use crate::error::Error;

use itertools::Itertools;


#[derive(Debug, Clone)]
pub struct Evaluator {
    pub context: Context,
    pub defining: Option<String>,
}

impl Evaluator {
    pub fn new() -> Self {
        Evaluator { context: Context::new(), defining: None }
    }

    pub fn eval_expr_mut_context(&mut self, expr: &Expr) -> CResult<Option<UnitVal>> {
        match expr {
            EDefVar(var, expr) => {
                if self.defining.is_some() {
                    return Err(Error::EvalError(format!("Cannot contain nested variable definitions (variable '{}' & '{}')", var, self.defining.as_ref().unwrap())));
                }
                self.defining = Some(var.clone());
                let result = self.eval_expr(expr)?;
                self.defining = None;
                if self.context.vars.contains_key(var) {
                    return Err(Error::EvalError(format!("Variable '{var}' already defined")));
                }
                self.context.vars.insert(var.clone(), result.clone());
                Ok(Some(result))
            },
            EDefFunc(name, params, expr) => {
                if self.defining.is_some() {
                    return Err(Error::EvalError(format!("Cannot contain nested variable definitions (variable '{}' & '{}')", name, self.defining.as_ref().unwrap())));
                }
                if self.context.funcs.contains_key(name) {
                    return Err(Error::EvalError(format!("Variable '{name}' already defined")));
                }
                self.context.funcs.insert(name.clone(), (params.clone(), *expr.clone()));
                Ok(None)
            },
            _ => Ok(Some(self.eval_expr(expr)?)),
        }
    }
 
    pub fn eval_expr(&self, expr: &Expr) -> CResult<UnitVal> {
    
        match expr {
            ENum(num) => Ok(num.clone()),
            EAdd(expr1, expr2) => self.eval_expr(expr1)? + self.eval_expr(expr2)?,
            ESub(expr1, expr2) => self.eval_expr(expr1)? - self.eval_expr(expr2)?,
            EMul(expr1, expr2) => Ok(self.eval_expr(expr1)? * self.eval_expr(expr2)?),
            EDiv(expr1, expr2) => Ok(self.eval_expr(expr1)? / self.eval_expr(expr2)?),
            EExp(expr1, expr2) => self.eval_expr(expr1)?.powf(self.eval_expr(expr2)?),
            EVar(var) => {
                if self.defining.is_some() && var == self.defining.as_ref().unwrap() {
                    return Err(Error::EvalError(format!("Variable '{var}' cannot be defined recursively")))
                } else if let Some(val) = self.context.vars.get(var) {
                    Ok(val.clone())
                } else {
                    Err(Error::EvalError(format!("Variable '{var}' not defined")))
                }
            },
            EFunc(name, inputs) => {
                if self.defining.is_some() && name == self.defining.as_ref().unwrap() {
                    Err(Error::EvalError(format!("Function '{name}' cannot be defined recursively")))
                } else if let Some((params, func_def)) = self.context.funcs.get(name) {
                    if params.len() != inputs.len() {
                        return Err(Error::EvalError(format!("Function '{}' expects {} arguments, but got {}", name, params.len(), inputs.len())));
                    }
                    let mut sub_eval = self.clone();
                    for (param, input) in params.iter().zip(inputs.iter()) {
                        sub_eval.context.vars.insert(param.clone(), self.eval_expr(input)?);
                    }
                    sub_eval.eval_expr(func_def)
                } else {
                    Err(Error::EvalError(format!("Function '{name}' not defined")))
                }
            },
            ETex(expr) => self.eval_latex(expr),
            _ => Err(Error::EvalError(format!("Unexpected expression '{expr:?}'. Did you mean to call `eval_mut_context_def`?")),)
        }
    }

    fn apply_default_function(&self, name: &str, inputs: &Vec<Expr>) -> CResult<UnitVal> {
        if inputs.len() != 1 {
            return Err(Error::EvalError(format!("Default functions only accept one argument, received {} for {name}", inputs.len())));
        }
        let input = self.eval_expr(inputs.get(0).unwrap())?.as_scalar()?;
        match name {
            "sin" => Ok(UnitVal::scalar(input.sin())),
            "cos" => Ok(UnitVal::scalar(input.cos())),
            "tan" => Ok(UnitVal::scalar(input.tan())),
            _ => Err(Error::EvalError(format!("Function '{name}' not defined")))
        }
    }

    fn eval_latex(&self, expr: &LatexExpr) -> CResult<UnitVal> {
        match expr.name.as_str() {
            "frac" => {
                if expr.params.len() != 2 {
                    return Err(Error::EvalError("frac expects 2 arguments".to_string()));
                }
                if expr.subscript.is_some() || expr.superscript.is_some() {
                    return Err(Error::EvalError("frac does not support subscripts or superscripts".to_string()));
                }
                let (num, denom) = expr.params.clone().into_iter().collect_tuple().unwrap();
                Ok(self.eval_expr(&num)? / self.eval_expr(&denom)?)
            },
            "sqrt" => {
                if expr.params.len() != 1 {
                    return Err(Error::EvalError("Square root expects 1 argument".to_string()));
                }
                if expr.subscript.is_some() || expr.superscript.is_some() {
                    return Err(Error::EvalError("Square root does not support subscripts or superscripts".to_string()));
                }
                let val = expr.params.get(0).unwrap();
                Ok(self.eval_expr(val)?.root(2)?)
            },
            "sum" => {
                self.evaluate_repetition(expr, |a, b| a + b, 0.0)
            },
            "prod" => {
                self.evaluate_repetition(expr, |a, b| a * b, 1.0)
            },
            name => {
                if expr.subscript.is_some() || expr.superscript.is_some() {
                    return Err(Error::EvalError(format!("Function {name} does not support subscripts or superscripts")));
                }
                self.apply_default_function(name, &expr.params)
            }
        }
    }

    fn evaluate_repetition(&self, expr: &LatexExpr, op: impl Fn(f32, f32) -> f32, identity: f32) -> Result<UnitVal, Error> {
        if expr.params.len() != 1 || expr.subscript.is_none() || expr.superscript.is_none() {
            return Err(Error::EvalError(format!("Summation expects a parameter, a subscript, and a superscript, received {:?}", expr)));
        }
        let param = expr.params.get(0).unwrap();
        let superscript = expr.superscript.as_ref().unwrap();
        let subscript = expr.subscript.as_ref().unwrap();
        let ub = self.eval_expr(&superscript)?;
    
        let mut sum_eval = self.clone();
        let lb_var = match *subscript.clone() {
            EDefVar(name, _) => Some(name),
            _ => None,
        };
        let lb = sum_eval.eval_expr_mut_context(&subscript)?.unwrap();

        // Ensure up and ub are integers
        if ub.fract()? != 0.0 || lb.fract()? != 0.0 {
            return Err(Error::EvalError("Summation bounds must be integers".to_string()));
        }
        let up = ub.as_scalar()? as i32 + 1;
        let ub = lb.as_scalar()? as i32;
    
        let mut sum = identity;
        for i in ub..up {
            if lb_var.is_some() {
                sum_eval.context.vars.insert(lb_var.clone().unwrap(), UnitVal::scalar(i as f32));
            }
            sum = op(sum, sum_eval.eval_expr(&param)?.as_scalar()?);
        }
        Ok(UnitVal::scalar(sum))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn evaluate(expr: Expr) -> UnitVal {
        let mut eval = Evaluator::new();
        eval.eval_expr_mut_context(&expr).unwrap().unwrap()
    }

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
        let mut eval = Evaluator::new();
        assert_eq!(eval.context.vars.get("a"), None);
        eval.eval_expr_mut_context(&expr).unwrap();
        assert_eq!(eval.context.vars.get("a"), Some(&UnitVal::scalar(2.0)));
    }

    #[test]
    fn test_function_definition_and_call() {
        // f(x,y) = x + y
        let expr = EDefFunc(
            "f".to_string(),
            vec!["x".to_string(), "y".to_string()],
            Box::new(EAdd(Box::new(EVar("x".to_string())), Box::new(EVar("y".to_string())))),
        );
        let mut eval = Evaluator::new();
        assert_eq!(eval.context.funcs.get("f"), None);
        eval.eval_expr_mut_context(&expr).unwrap();
        assert_ne!(eval.context.funcs.get("f"), None);

        let call = EFunc("f".to_string(), vec![num(1.0), num(2.0)]);
        let unit_val = eval.eval_expr_mut_context(&call).unwrap().unwrap();
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
