use pest::Parser;
use pest_derive::*;

#[derive(Parser)]
#[grammar = "extract/planner.pest"]
pub struct BilancioParser;

use std::fs;

pub fn extract(filename: &str) -> Option<Ast> {
    println!("In file {}", filename);

    let contents = match fs::read_to_string(filename) {
        Ok(s) => s,
        Err(_) => {
            println!("File not found {}", filename);
            return None;
        }
    };

    println!("With text:\n{}", contents);
    let contents = match BilancioParser::parse(Rule::program, &contents) {
        Ok(contents) => contents,
        Err(error) => {
            println!("{}", error);
            return None;
        }
    };
    println!("{:#?}", contents);
    None
}

pub enum Ast {
    Entries(Vec<Entry>),
    Templates(Vec<Template>),
}
