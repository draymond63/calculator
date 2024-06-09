use evalexpr::{eval_with_context, Value, context_map};
use Value::{Float, Int};

fn identify_latex_functions(input: &str) -> &str {
    input.replace("{", "(").replace("}", ")")
}

fn main() {
    let input = "sin(0.5) + 3 * \\frac(4, 2) + avg(3, 4)";
    // let input = identify_latex_functions(input);

    let context = context_map! {
        "\\frac" => Function::new(|argument| {
            let arguments = argument.as_tuple()?;
            Ok(Float(arguments[0].as_number()? / arguments[1].as_number()?))
        }),
        "sin" => Function::new(|argument| {
            Ok(Float(argument.as_number()?.sin()))
        }),
        "avg" => Function::new(|argument| {
            let arguments = argument.as_tuple()?;

            if let (Int(a), Int(b)) = (&arguments[0], &arguments[1]) {
                Ok(Int((a + b) / 2))
            } else {
                Ok(Float((arguments[0].as_number()? + arguments[1].as_number()?) / 2.0))
            }
        })
    }.unwrap();

    let result = eval_with_context(input, &context).unwrap();
    println!("{}", result);
}
