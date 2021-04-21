use pest::Parser;
use pest_derive::*;

#[derive(Parser)]
#[grammar = "bilancio.pest"]
pub struct BilancioParser;

use std::fs;

use crate::extract::validate;

pub fn extract(filename: &str) -> validate::Result<validate::Ast> {
    //println!("In file {}", filename);

    let contents = match fs::read_to_string(filename) {
        Ok(s) => s,
        Err(_) => {
            panic!("File not found {}", filename);
        }
    };

    //println!("With text:\n{}", contents);
    let contents = BilancioParser::parse(Rule::program, &contents)?;
    validate::validate(contents)
}
