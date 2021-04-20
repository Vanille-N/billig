#[derive(Debug)]
pub struct Instance {
    label: String,
    pos: Vec<Arg>,
    named: Vec<(String, Arg)>,
}

use crate::extract::{
    Amount,
    Tag,
};

#[derive(Debug)]
pub enum Arg {
    Amount(Amount),
    Tag(Tag),
}

