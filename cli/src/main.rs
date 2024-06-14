use evalexpr::{build_operator_tree, context_map, eval_with_context_mut, ContextWithMutableFunctions, ContextWithMutableVariables, EvalexprError, Function, HashMapContext, Value};
use Value::{Float, Empty};
use regex::Regex;


fn replace_latex_functions(input: &str) -> String {
    input.replace("}{", ",")
         .replace("{", "(")
         .replace("}", ")")
}

fn eval_user_function(input: &String, global_context: &mut HashMapContext) -> bool {
    let re = Regex::new(r"^([a-zA-Z]+)\(([^)]+)\)\s*=").unwrap();
    let captures = re.captures(input);

    if captures.is_none() {
        return false;
    }
    let captures = captures.unwrap();

    if captures.len() == 3 {
        let full_capture = &captures[0];
        let function_name = &captures[1];
        let arg_str = &captures[2];
        let equation = &input[full_capture.len()..];
        let function = compile_function(arg_str, equation, global_context);
        global_context.set_function(function_name.to_string(), function).unwrap();
        true
    } else {
        false // TODO: Should be an error
    }
}

fn compile_function(arg_str: &str, equation: &str, global_context: &mut HashMapContext) -> Function {
    let arg_vars = arg_str.split(",")
                    .map(|s| s.trim().to_string())
                    .collect::<Vec<String>>();
    let precompiled = build_operator_tree(equation).unwrap();
    let context = global_context.clone();

    Function::new(move |argument: &Value| {
        let mut context = context.to_owned();
        let arguments = if argument.is_tuple() { argument.as_tuple()? } else { vec![argument.clone()] };
        if arguments.len() != arg_vars.len() {
            return Err(EvalexprError::wrong_function_argument_amount(arguments.len(), arg_vars.len()));
        }
        for (i, arg) in arguments.iter().enumerate() {
            context.set_value(arg_vars[i].to_string(), arg.clone()).unwrap();
        }
        precompiled.eval_with_context(&context)
    })
}

fn evaluate_expression(input: &str, context: &mut HashMapContext) -> Result<Value, EvalexprError> {
    let input = &replace_latex_functions(input);
    if eval_user_function(input, context) {
        Ok(Empty)
    } else {
        eval_with_context_mut(input, context)
    }
}

fn get_base_context() -> HashMapContext {
    context_map! {
        "\\frac" => Function::new(|argument| {
            let arguments = argument.as_tuple()?;
            let numerator = &arguments[0];
            let denom = &arguments[1];
            Ok(Float(numerator.as_number()? / denom.as_number()?))
        }),
        "sin" => Function::new(|argument| {
            Ok(Float(argument.as_number()?.sin()))
        }),
        "avg" => Function::new(|argument| {
            let arguments = &argument.as_tuple()?;
            let mut sum = 0.0;
            for arg in arguments {
                sum += arg.as_number()?;
            }
            Ok(Float(sum / arguments.len() as f64))
        }),
    }.unwrap()
}

fn compute(inputs: Vec<&str>) -> Result<Vec<Value>, EvalexprError> {
    let mut context = get_base_context();

    let results = inputs.into_iter()
                        .map(|input| evaluate_expression(input, &mut context))
                        .collect::<Result<_, _>>()?;
    Ok(results)
}

fn main() {
    let inputs = vec![
        "a = 16",
        "b = 2",
        "f(x, y) = x / y^2",
        "g(x) = f(x, b)",
        "g(a)",
    ];
    let results = compute(inputs).unwrap();
    println!("Results: {:?}", results);
}
