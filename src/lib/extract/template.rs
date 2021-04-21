use crate::extract::{
    entry::{Category, Span},
    instance::Arg,
    Amount, 
};

#[derive(Debug)]
pub struct Template<'i> {
    pub positional: Vec<&'i str>,
    pub named: Vec<(&'i str, Arg<'i>)>,
    pub value: AmountTemplate<'i>,
    pub cat: Category,
    pub span: Span,
    pub tag: TagTemplate<'i>,
}

#[derive(Debug)]
pub struct TagTemplate<'i>(pub Vec<TagTemplateItem<'i>>);

#[derive(Debug)]
pub enum TagTemplateItem<'i> {
    Day,
    Month,
    Year,
    Date,
    Weekday,
    Raw(&'i str),
    Arg(&'i str),
}

#[derive(Debug)]
pub struct AmountTemplate<'i> {
    pub sign: bool,
    pub sum: Vec<AmountTemplateItem<'i>>,
}

#[derive(Debug)]
pub enum AmountTemplateItem<'i> {
    Cst(Amount),
    Arg(&'i str),
}
