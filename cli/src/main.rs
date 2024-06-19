use crate::{
    evaluator::eval_mut_context,
    types::Context,
    parser::parse,
};

use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::result::Result;

mod evaluator;
mod parser;
mod types;



fn evaluate(inputs: Vec<String>) -> Result<Vec<f32>, Box<dyn Error>> {
    let mut context = Context::new();
    let mut results = vec![0.0; inputs.len()];
    for (i, input) in inputs.into_iter().enumerate() {
        let expr = parse(&input)?;
        results[i] = eval_mut_context(expr, &mut context);
    }
    Ok(results)
}




fn main() -> Result<(), Box<dyn Error>> {
    let mut test_file = File::open("test.bc")?;
    let mut input_file_contents = String::new();
    test_file.read_to_string(&mut input_file_contents)?;
    let inputs = input_file_contents.lines()
                        .map(String::from)
                        .collect::<Vec<String>>();
    println!("{:#?}", evaluate(inputs)?);
    Ok(())
}
