use crate::extract::{
    entry::{Category, Span},
    instance::Arg,
    Amount, Tag,
};

#[derive(Debug)]
pub struct Template {
    pub arguments: Vec<(String, Option<Arg>)>,
    pub value: AmountTemplate,
    pub cat: Category,
    pub span: Span,
    pub tag: TagTemplate,
}

#[derive(Debug)]
pub struct TagTemplate(pub Vec<TagTemplateItem>);

#[derive(Debug)]
pub enum TagTemplateItem {
    Day,
    Month,
    Year,
    Raw(Tag),
    Arg(String),
}

#[derive(Debug)]
pub struct AmountTemplate {
    pub sign: bool,
    pub sum: Vec<AmountTemplateItem>,
}

#[derive(Debug)]
pub enum AmountTemplateItem {
    Cst(Amount),
    Arg(String),
}
