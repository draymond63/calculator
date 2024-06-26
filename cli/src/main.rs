
use crate::{
    evaluator::eval_mut_context,
    types::{Context, Span},
    parser::parse,
    units::UnitVal,
};

use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::result::Result;
use std::convert::TryInto;

mod evaluator;
mod parser;
mod parsing_helpers;
mod types;
mod units;



fn evaluate(inputs: Vec<&str>) -> Result<Vec<Option<UnitVal>>, Box<dyn Error>> {
    let inputs = inputs.into_iter().filter(|s| !s.is_empty()).collect::<Vec<&str>>();
    let mut context = Context::new();
    let mut results = vec![None; inputs.len()];

    for (i, input) in inputs.into_iter().enumerate() {        
        let line_num: u32 = (i + 1).try_into().unwrap();
        let line = unsafe { Span::new_from_raw_offset(0, line_num, &input, ()) };
        let (_, expr) = parse(line).unwrap();
        println!("Parsed: {:?}", expr);

        let eval = eval_mut_context(&expr, &mut context);
        if eval.is_err() {
            return Err(format!("Failed to evaluate: {:?}", eval.unwrap_err()).into());
        }
        let res = eval.unwrap();
        results[i] = res.clone();
        let unit_val = res.unwrap();
        println!("{} = {:?}", input, unit_val.to_string());
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
