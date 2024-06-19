use crate::{
    evaluator::eval_mut_context,
    types::Context,
    parser::parse,
};

use nom::error::convert_error;

use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::result::Result;

mod evaluator;
mod parser;
mod types;



fn evaluate(inputs: Vec<&str>) -> Result<Vec<f32>, Box<dyn Error>> {
    let inputs = inputs.into_iter().filter(|s| !s.is_empty()).collect::<Vec<&str>>();
    let mut context = Context::new();
    let mut results = vec![0.0; inputs.len()];

    for (i, input) in inputs.into_iter().enumerate() {
        let result = parse(&input);

        let expr = match result {
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                panic!("Could not parse - {}", convert_error(input, e));
            }
            Ok((_, expr)) => expr,
            k => {
                panic!("unexpected parse error {:?}", k);
            }
        };

        results[i] = eval_mut_context(expr, &mut context);
        println!("{} = {}", input, results[i])
    }
    Ok(results)
}



fn main() -> Result<(), Box<dyn Error>> {
    let mut test_file = File::open("test.bc")?;
    let mut input_file_contents = String::new();
    test_file.read_to_string(&mut input_file_contents)?;
    let inputs = input_file_contents.lines().collect::<Vec<&str>>();
    evaluate(inputs)?;
    Ok(())
}
