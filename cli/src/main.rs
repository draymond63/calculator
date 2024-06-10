use evalexpr::{eval_with_context_mut, Value, context_map, EvalexprError};
use Value::Float;

fn identify_latex_functions(input: &str) -> String {
    input.replace("}{", ",")
         .replace("{", "(")
         .replace("}", ")")
}

fn compute(inputs: Vec<&str>) -> Result<Vec<Value>, EvalexprError> {
    let inputs = inputs.into_iter()
                        .map(identify_latex_functions)
                        .collect::<Vec<String>>();
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
        })
    }.unwrap();

    let mut results: Vec<Value> = Vec::new();
    for input in inputs.iter() {
        results.push(eval_with_context_mut(input, &mut context).unwrap());
    }
    Ok(results)
}

fn main() {
    let inputs = vec![
        "a = 1/2",
        "sin(a) + \\frac{a}{2}"
    ];

    let results = compute(inputs).unwrap();
    for result in results {
        println!("{}", result);
    }
}
