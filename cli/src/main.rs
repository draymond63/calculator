use evalexpr::{build_operator_tree, context_map, eval_with_context_mut, ContextWithMutableFunctions, ContextWithMutableVariables, EvalexprError, Function, HashMapContext, Value};
use Value::Float;
use regex::Regex;

fn identify_latex_functions(input: &str) -> String {
    input.replace("}{", ",")
         .replace("{", "(")
         .replace("}", ")")
}


fn compile_user_function(input: &String, global_context: &mut HashMapContext) -> bool {
    let re = Regex::new(r"^([a-zA-Z]+)\(([^)]+)\)\s*=").unwrap();
    let captures = re.captures(input);

    if captures.is_none() {
        return false;
    }
    let captures = captures.unwrap();

    if captures.len() == 3 {
        let full_capture = &captures[0];
        let function_name = String::from(&captures[1]);
        let arg_var = String::from(&captures[2]);        
        let equation = &input[full_capture.len()..];
        // let arguments = arguments.split(",").collect::<Vec<&str>>();
        let precompiled = build_operator_tree(equation).unwrap();
        let context = global_context.clone();

        let function = Function::new(move |argument: &Value| {
            let mut context = context.to_owned();
            context.set_value(arg_var.clone(), argument.clone())?;
            precompiled.eval_with_context(&context)
        });
        global_context.set_function(function_name, function).unwrap();
        true
    } else {
        false
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

    let inputs = inputs.into_iter()
                        .map(identify_latex_functions)
                        .collect::<Vec<String>>();

    let mut results: Vec<Value> = Vec::new();
    for input in inputs.iter() {
        if !compile_user_function(input, &mut context) {
            results.push(eval_with_context_mut(input, &mut context).unwrap());
        }
    }
    Ok(results)
}

fn main() {
    let inputs = vec![
        "a = 3",
        "b = 2",
        "f(x) = x^3",
        "g(x) = 2 + f(x)/b",
        "g(a)",
    ];
    let results = compute(inputs).unwrap();
    println!("Results: {:?}", results);
}
