use pest::Parser;
use pest_derive::*;

#[derive(Parser)]
#[grammar = "bilancio.pest"]
pub struct BilancioParser;

use crate::extract::validate;

pub fn extract(contents: &str) -> validate::Result<validate::Ast> {
    let contents = BilancioParser::parse(Rule::program, contents)?;
    validate::validate(contents)
}
