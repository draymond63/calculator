use crate::{
    evaluator::eval_mut_context,
    types::{Context, Span},
    parser::parse,
};

use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::result::Result;
use std::convert::TryInto;

mod evaluator;
mod parser;
mod types;



fn evaluate(inputs: Vec<&str>) -> Result<Vec<f32>, Box<dyn Error>> {
    let inputs = inputs.into_iter().filter(|s| !s.is_empty()).collect::<Vec<&str>>();
    let mut context = Context::new();
    let mut results = vec![0.0; inputs.len()];

    for (i, input) in inputs.into_iter().enumerate() {        
        let line_num: u32 = (i + 1).try_into().unwrap();
        let line = unsafe { Span::new_from_raw_offset(0, line_num, &input, ()) };
        let result = parse(line);

        let expr = match result {
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                return Err(format!("Failed to parse: {:?}", e).into());
            }
            Ok((_, expr)) => expr,
            k => {
                panic!("Unexpected parse error {:?}", k);
            }
        };

        let eval = eval_mut_context(expr, &mut context);
        if eval.is_err() {
            return Err(format!("Failed to evaluate: {:?}", eval.unwrap_err()).into());
        }
        results[i] = eval.unwrap();
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
