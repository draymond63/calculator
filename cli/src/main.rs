use evalexpr::{build_operator_tree, context_map, eval_with_context_mut, ContextWithMutableFunctions, ContextWithMutableVariables, EvalexprError, Function, HashMapContext, Value};
use Value::Float;
use regex::Regex;

fn identify_latex_functions(input: &str) -> String {
    input.replace("}{", ",")
         .replace("{", "(")
         .replace("}", ")")
}


fn compile_user_function(input: &String, global_context: &HashMapContext) -> Option<(String, Function)> {
    let re = Regex::new(r"^([a-zA-Z]+)\(([^)]+)\)\s*=").unwrap();
    let captures = re.captures(input)?;

    if captures.len() == 3 {
        let full_capture = &captures[0];
        let function_name = String::from(&captures[1]);
        let arg_var = String::from(&captures[2]);        
        let equation = &input[full_capture.len()..];
        // let arguments = arguments.split(",").collect::<Vec<&str>>();
        let precompiled = build_operator_tree(equation).unwrap();
        let context = global_context.clone();
        println!("Context: {:?}", context);

        let equation = Function::new(move |argument: &Value| {
            let mut context = context.to_owned();
            context.set_value(arg_var.clone(), argument.clone())?;
            precompiled.eval_with_context(&context)
        });
        Some((function_name, equation))
    } else {
        None
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

    let mut inputs = inputs.into_iter()
                        .map(identify_latex_functions)
                        .collect::<Vec<String>>();

    let mut user_func_indices: Vec<usize> = Vec::new();
    for (i, input) in inputs.iter().enumerate() {
        let user_func = compile_user_function(input, &context);
        if user_func.is_some() {
            let (name, function) = user_func.unwrap();
            println!("Adding function: {}", name);
            context.set_function(name, function)?;
            user_func_indices.push(i);
        }
    }
    user_func_indices.reverse();
    for i in user_func_indices {
        println!("Removing: {}", i);
        inputs.remove(i);
    }
    println!("Inputs: {:?}", inputs);

    let mut results: Vec<Value> = Vec::new();
    for input in inputs.iter() {
        results.push(eval_with_context_mut(input, &mut context).unwrap());
    }
    Ok(results)
}

fn main() {
    let inputs = vec![
        "a = 3",
        "b = 2",
        "f(x) = x^3",
        "g(x) = 2 + \\frac{f(x)}{2}",
        "g(a)",
    ];
    let results = compute(inputs).unwrap();
    println!("Results: {:?}", results);
}
