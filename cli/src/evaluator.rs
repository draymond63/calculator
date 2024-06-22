use crate::types::{
    Expr,
    Expr::*,
    Context,
};

pub(crate) fn eval_mut_context(expr: &Expr, mut context: &mut Context) -> Result<Option<f32>, String> {
    eval_mut_context_def(expr, &mut context, None)
}

fn eval_mut_context_def(expr: &Expr, mut context: &mut Context, defining: Option<&str>) -> Result<Option<f32>, String> {
    match expr {
        EDefVar(var, expr) => {
            let result = eval_mut_context_def(expr, &mut context, Some(&var))?.unwrap();
            if defining.is_some() {
                return Err(format!("Cannot contain nested variable definitions (variable '{}' & '{}')", var, defining.unwrap()));
            }
            if context.vars.contains_key(var) {
                return Err(format!("Variable '{var}' already defined"));
            }
            context.vars.insert(var.clone(), result);
            Ok(None)
        }
        EDefFunc(name, params, expr) => {
            if defining.is_some() {
                return Err(format!("Cannot contain nested variable definitions (variable '{}' & '{}')", name, defining.unwrap()));
            }
            if context.funcs.contains_key(name) {
                return Err(format!("Variable '{name}' already defined"));
            }
            context.funcs.insert(name.clone(), (params.clone(), *expr.clone()));
            Ok(None)
        },
        _ => eval_expr(expr, &context, defining),
    }
}

fn eval_expr(expr: &Expr, context: &Context, defining: Option<&str>) -> Result<Option<f32>, String> {
    let compose = |expr1: &Expr, expr2: &Expr, func: fn(f32, f32) -> f32| -> Result<Option<f32>, String> {
        Ok(Some(func(eval_expr(expr1, context, defining)?.unwrap(), eval_expr(expr2, context, defining)?.unwrap())))
    };

    match expr {
        ENum(num) => Ok(Some(*num)),
        EAdd(expr1, expr2) => compose(expr1, expr2, |a, b| a + b),
        ESub(expr1, expr2) => compose(expr1, expr2, |a, b| a - b),
        EMul(expr1, expr2) => compose(expr1, expr2, |a, b| a * b),
        EDiv(expr1, expr2) => compose(expr1, expr2, |a, b| a / b),
        EExp(expr1, expr2) => compose(expr1, expr2, |a, b| a.powf(b)),
        EVar(var) => {
            if defining.is_some() && var == defining.unwrap() {
                return Err(format!("Variable '{var}' cannot be defined recursively"))
            } else if let Some(val) = context.vars.get(var) {
                Ok(Some(*val))
            } else {
                Err(format!("Variable '{var}' not defined"))
            }
        },
        EFunc(name, inputs) => {
            if defining.is_some() && name == defining.unwrap() {
                return Err(format!("Function '{name}' cannot be defined recursively"))
            } else if let Some((params, func_def)) = context.funcs.get(name) {
                if params.len() != inputs.len() {
                    return Err(format!("Function '{}' expects {} arguments, but got {}", name, params.len(), inputs.len()));
                }
                let mut eval_context = context.clone();

                for (param, input) in params.iter().zip(inputs.iter()) {
                    eval_context.vars.insert(param.clone(), eval_expr(input, context, defining)?.unwrap());
                }
                eval_mut_context_def(func_def, &mut eval_context, defining)
            } else {
                Err(format!("Function '{name}' not defined"))
            }
        },
        _ => Err(format!("Unexpected expression '{expr:?}'. Did you mean to call `eval_mut_context_def`?")),
    }
}


#[cfg(test)]
pub(crate) fn evaluate(expr: Expr) -> f32 {
    let mut context = Context::new();
    eval_mut_context(&expr, &mut context).unwrap().unwrap()
}

#[cfg(test)]
mod tests {
    use crate::evaluator::{evaluate, eval_mut_context_def};
    use crate::types::{Expr::*, Context};

    #[test]
    fn evaluate_enum_test() {
        let expr = ENum(1234.0);
        assert_eq!(evaluate(expr), 1234.0);
    }

    #[test]
    fn evaluate_eadd_test() {
        let expr = EAdd(Box::new(ENum(12.0)), Box::new(ENum(34.0)));
        assert_eq!(evaluate(expr), 46.0);
    }

    #[test]
    fn evaluate_easub_test() {
        let expr = ESub(Box::new(ENum(12.0)), Box::new(ENum(34.0)));
        assert_eq!(evaluate(expr), -22.0);
    }

    #[test]
    fn test_evaluate_nested_arithmetic_expression() {
        let expr = EAdd(
            Box::new(EMul(Box::new(ENum(1.0)), Box::new(ENum(2.0)))),
            Box::new(EDiv(
                Box::new(EExp(Box::new(ENum(6.0)), Box::new(ENum(2.0)))),
                Box::new(ENum(5.0)),
            )),
        );
        assert_eq!(evaluate(expr), 9.2);
    }

    #[test]
    fn test_variable_definition() {
        let expr = EDefVar(
            "a".to_string(),
            Box::new(ENum(2.0)),
        );
        let mut context = Context::new();
        assert_eq!(context.vars.get("a"), None);
        eval_mut_context_def(&expr, &mut context, None).unwrap();
        assert_eq!(context.vars.get("a"), Some(&2.0));
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

        let call = EFunc("f".to_string(), vec![ENum(1.0), ENum(2.0)]);
        assert_eq!(eval_mut_context_def(&call, &mut context, None).unwrap().unwrap(), 3.0);
    }
}
