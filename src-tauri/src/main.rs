// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::{
  evaluator::eval_mut_context,
  types::{Context, Span},
  parser::parse,
  units::UnitVal,
};

use std::env;
use std::fs::File;
use std::io::Read;
use std::result::Result;

mod evaluator;
mod parser;
mod parsing_helpers;
mod types;
mod units;


type EvalResult = Result<Option<UnitVal>, String>;


fn evaluate_line(line: nom_locate::LocatedSpan<&str>, context: &mut Context) -> EvalResult {
    let parse_res = parse(line);
    if let Ok((_, expr)) = parse_res {
        println!("Parsed: {:?}", expr);
        let eval = eval_mut_context(&expr, context);
        let res = eval?;
        println!("{} = {:?}", line.fragment(), res);
        Ok(res)
    } else {
        Err(format!("Failed to parse: {:?}", parse_res.unwrap_err()).into())
    }
}

fn evaluate_sequence(inputs: Vec<&str>) -> Vec<EvalResult> {
    let inputs = inputs.into_iter().filter(|s| !s.is_empty()).collect::<Vec<&str>>();
    let mut context = Context::new();
    let mut results = vec![];

    for (i, input) in inputs.into_iter().enumerate() {        
        let line_num: u32 = (i + 1) as u32;
        let line = unsafe { Span::new_from_raw_offset(0, line_num, &input, ()) };
        results.push(evaluate_line(line, &mut context));
    }
    results
}

#[tauri::command]
async fn evaluate(input: &str) -> Result<Vec<EvalResult>, ()> {
    let inputs = input.lines().collect::<Vec<&str>>();
    Ok(evaluate_sequence(inputs))
}


fn main() {
  let args: Vec<String> = env::args().collect();
  let file_path = &args.get(1);

  if file_path.is_some() {
    let mut test_file = File::open(file_path.unwrap()).unwrap();
    let mut input_file_contents = String::new();
    test_file.read_to_string(&mut input_file_contents).unwrap();
    let inputs = input_file_contents.lines().collect::<Vec<&str>>();
    evaluate_sequence(inputs);
  } else {
    tauri::Builder::default()
      .invoke_handler(tauri::generate_handler![evaluate])
      .run(tauri::generate_context!())
      .expect("error while running tauri application");
  }
}
