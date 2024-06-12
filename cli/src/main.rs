use evalexpr::{build_operator_tree, context_map, eval_with_context_mut, ContextWithMutableFunctions, EvalexprError, Function, Value};
use Value::Float;
use regex::Regex;

fn identify_latex_functions(input: &str) -> String {
    input.replace("}{", ",")
         .replace("{", "(")
         .replace("}", ")")
}


fn compile_user_function(input: &String) -> Option<(String, Function)> {
    let re = Regex::new(r"^([a-zA-Z]+)\(([^)]+)\)\s*=").unwrap();
    let captures = re.captures(input)?;

    if captures.len() == 3 {
        let full_capture = &captures[0];
        let function_name = String::from(&captures[1]);
        let arg_var = String::from(&captures[2]);        
        let equation = &input[full_capture.len()..];
        // let arguments = arguments.split(",").collect::<Vec<&str>>();
        let precompiled = build_operator_tree(equation).unwrap();

        let equation = Function::new(move |argument: &Value| {
            let context = context_map! {
                arg_var.clone() => argument.clone()
            }?;
            println!("{arg_var} = {argument}");
            precompiled.eval_with_context(&context)
        });
        Some((function_name, equation))
    } else {
        None
    }
}

fn compute(inputs: Vec<&str>) -> Result<Vec<Value>, EvalexprError> {
    let mut inputs = inputs.into_iter()
                        .map(identify_latex_functions)
                        .collect::<Vec<String>>();
    let user_funcs = inputs.iter()
                           .map(compile_user_function)
                           .collect::<Vec<Option<(String, Function)>>>();
    let user_func_indices = user_funcs.iter()
                                      .enumerate()
                                      .filter(|(_, x)| x.is_some())
                                      .map(|(i, _)| i)
                                      .collect::<Vec<usize>>();
    
    let user_funcs = user_funcs.into_iter()
                                 .filter(|x| x.is_some())
                                 .map(|x| x.unwrap())
                                 .collect::<Vec<(String, Function)>>();

    println!("Funcs: {:?}", user_funcs);
    let mut context = context_map! {
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
    }.unwrap();

    // Add user functions to context
    for (name, function) in user_funcs {
        context.set_function(name, function)?;
    }
    for i in user_func_indices {
        inputs.remove(i);
    }

    let mut results: Vec<Value> = Vec::new();
    for input in inputs.iter() {
        results.push(eval_with_context_mut(input, &mut context).unwrap());
    }
    Ok(results)
}

fn main() {
    let inputs = vec![
        "a = 0.5",
        "f(x) = x^3",
        "f(a)",
    ];

    let results = compute(inputs).unwrap();
    println!("Results: {:?}", results);
}
