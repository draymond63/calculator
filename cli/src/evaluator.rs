use crate::types::{
    Expr,
    Expr::*,
    Context,
};



pub(crate) fn eval_mut_context(expr: Expr, mut context: &mut Context) -> f32 {
    match expr {
        ENum(num) => num,
        EAdd(expr1, expr2) => eval_mut_context(*expr1, &mut context) + eval_mut_context(*expr2, &mut context),
        ESub(expr1, expr2) => eval_mut_context(*expr1, &mut context) - eval_mut_context(*expr2, &mut context),
        EMul(expr1, expr2) => eval_mut_context(*expr1, &mut context) * eval_mut_context(*expr2, &mut context),
        EDiv(expr1, expr2) => eval_mut_context(*expr1, &mut context) / eval_mut_context(*expr2, &mut context),
        EExp(expr1, expr2) => eval_mut_context(*expr1, &mut context).powf(eval_mut_context(*expr2, &mut context)),
        EVar(var) => *context.vars.get(&var).expect("Variable not found"),
        EFunc(_, _) => panic!("Function not implemented"),
        EDefVar(var, expr) => {
            let result = eval_mut_context(*expr, &mut context);
            context.vars.insert(var, result);
            result
        }
        EDefFunc(_, _, _) => panic!("Function not implemented"),
    }
}

pub(crate) fn evaluate(expr: Expr) -> f32 {
    let mut context = Context::new();
    eval_mut_context(expr, &mut context)
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
