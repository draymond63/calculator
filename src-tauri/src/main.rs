// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::{
  evaluator::Evaluator,
  types::{Span, CResult},
  parser::parse,
  unit_value::UnitVal,
  menus::{save_file, get_menus, handle_menu_event},
};

use std::env;
use std::fs::File;
use std::io::Read;
use std::result::Result;

mod evaluator;
mod parser;
mod parsing_helpers;
mod types;
mod unit_value;
mod units;
mod error;
mod menus;


type EvalResult = CResult<Option<UnitVal>>;


fn evaluate_line(line: nom_locate::LocatedSpan<&str>, eval: &mut Evaluator) -> EvalResult {
    let expr = parse(line)?;
    println!("Parsed: {:?}", expr);
    let eval = eval.eval_expr_mut_context(&expr)?;
    println!("{} = {:?}", line.fragment(), eval);
    Ok(eval)
}

fn evaluate_sequence(inputs: Vec<&str>) -> Vec<EvalResult> {
    let mut eval = Evaluator::new();
    let mut results = vec![];

    for (i, input) in inputs.into_iter().enumerate() {      
        if input.is_empty() {
            results.push(Ok(None));
        } else {
            let line_num: u32 = (i + 1) as u32;
            let line = unsafe { Span::new_from_raw_offset(0, line_num, &input, ()) };
            results.push(evaluate_line(line, &mut eval));
        }
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
    for (i, result) in evaluate_sequence(inputs.clone()).iter().enumerate() {
      if let Ok(Some(val)) = result {
        println!("{} = {}", inputs[i], val);
      } else if let Err(err) = result {
        println!("Error: {:?}", err);
      }
    }
  } else {
    tauri::Builder::default()
      .menu(get_menus())
      .on_menu_event(handle_menu_event)
      .invoke_handler(tauri::generate_handler![evaluate, save_file])
      .run(tauri::generate_context!())
      .expect("error while running tauri application");
  }
}




#[cfg(test)]
mod tests {
    use crate::{Evaluator, Span, evaluate_line};

    #[test]
    fn valid_input() {
        let valid_inputs = vec![
            "(1 km^3 + 300 m^3)^(1/3)",
            "f\\left(x\\right)=\\sum_{i=1}^3x^i",
        ];
        let mut eval = Evaluator::new();
        for input in valid_inputs.into_iter() {      
            let line = Span::new(&input);
            evaluate_line(line, &mut eval).unwrap();
        }
    }

    #[test]
    fn invalid_input() {
        let invalid_inputs = vec![
            "f(2x)=x",
        ];
        let mut eval = Evaluator::new();
        for input in invalid_inputs.into_iter() {      
            let line = Span::new(&input);
            evaluate_line(line, &mut eval).unwrap_err();
        }
    }
}
