use crate::types::{
    Expr,
    Expr::*,
    Context,
};

pub(crate) fn eval_mut_context(expr: Expr, mut context: &mut Context) -> Result<f32, String> {
    eval_mut_context_def(expr, &mut context, None)
}

fn eval_mut_context_def(expr: Expr, mut context: &mut Context, defining: Option<&str>) -> Result<f32, String> {
    match expr {
        ENum(num) => Ok(num),
        EAdd(expr1, expr2) => Ok(eval_mut_context_def(*expr1, &mut context, defining)? + eval_mut_context_def(*expr2, &mut context, defining)?),
        ESub(expr1, expr2) => Ok(eval_mut_context_def(*expr1, &mut context, defining)? - eval_mut_context_def(*expr2, &mut context, defining)?),
        EMul(expr1, expr2) => Ok(eval_mut_context_def(*expr1, &mut context, defining)? * eval_mut_context_def(*expr2, &mut context, defining)?),
        EDiv(expr1, expr2) => Ok(eval_mut_context_def(*expr1, &mut context, defining)? / eval_mut_context_def(*expr2, &mut context, defining)?),
        EExp(expr1, expr2) => Ok(eval_mut_context_def(*expr1, &mut context, defining)?.powf(eval_mut_context_def(*expr2, &mut context, defining)?)),
        EVar(var) => {
            if defining.is_some() && var == defining.unwrap() {
                return Err(format!("Variable '{var}' cannot be defined recursively"))
            } else if let Some(val) = context.vars.get(&var) {
                Ok(*val)
            } else {
                Err(format!("Variable '{var}' not defined"))
            }
        },
        // EFunc(_, _) => panic!("Function not implemented"),
        EDefVar(var, expr) => {
            let result = eval_mut_context_def(*expr, &mut context, Some(&var))?;
            if defining.is_some() {
                return Err(format!("Cannot contain nested variable definitions (variable '{}' & '{}')", var, defining.unwrap()));
            }
            if context.vars.contains_key(&var) {
                return Err(format!("Variable '{var}' already defined"));
            }
            context.vars.insert(var, result);
            Ok(result)
        }
        // EDefFunc(_, _, _) => panic!("Function not implemented"),
    }
}

#[cfg(test)]
pub(crate) fn evaluate(expr: Expr) -> f32 {
    let mut context = Context::new();
    eval_mut_context(expr, &mut context).unwrap()
}

#[cfg(test)]
mod tests {
    use crate::evaluator::evaluate;
    use crate::types::Expr::*;

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
}
