use pest::Parser;
use pest_derive::*;

#[derive(Parser)]
#[grammar = "planner.pest"]
pub struct PlanParser;

use std::fs;

pub fn main() {
    let filename = "data.pln";
    println!("In file {}", filename);

    let contents = fs::read_to_string(filename)
        .expect("Something went wrong reading the file");

    println!("With text:\n{}", contents);
    println!("{:#?}", PlanParser::parse(Rule::program, &contents));
}
