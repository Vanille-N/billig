#[derive(Debug)]
pub struct Instance<'i> {
    pub label: &'i str,
    pub pos: Vec<Arg<'i>>,
    pub named: Vec<(&'i str, Arg<'i>)>,
}

use crate::extract::Amount;

#[derive(Debug, Clone, Copy)]
pub enum Arg<'i> {
    Amount(Amount),
    Tag(&'i str),
}
