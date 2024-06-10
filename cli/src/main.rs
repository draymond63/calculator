use evalexpr::{eval_with_context, Value, context_map, EvalexprError};
use Value::Float;

fn identify_latex_functions(input: &str) -> String {
    input.replace("}{", ",")
         .replace("{", "(")
         .replace("}", ")")
}


fn compute(input: &str) -> Result<Value, EvalexprError> {
    let input = identify_latex_functions(input);
    let allowed_functions = context_map! {
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

    return eval_with_context(&input, &allowed_functions);
}

fn main() {
    let input = "sin(0.5) + 3 * \\frac{4}{2} + avg(3, 4, 2)";

    println!("{}", compute(input).unwrap());
}
