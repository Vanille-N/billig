#[derive(Debug)]
pub struct Instance {
    pub label: String,
    pub pos: Vec<Arg>,
    pub named: Vec<(String, Arg)>,
}

use crate::extract::{Amount, Tag};

#[derive(Debug)]
pub enum Arg {
    Amount(Amount),
    Tag(Tag),
}
