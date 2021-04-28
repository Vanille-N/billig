//! Instanciate templates with their arguments
//!
//! Performs string replacements and concatenations,
//! as well as summations of amounts.
//!
//! Some minimal type checking involved as well.

use std::collections::{HashMap, HashSet};

use crate::lib::{
    date::Date,
    entry::{
        fields::{self, Category, Span},
        Entry,
    },
};
use crate::load::{error, parse::ast};

/// Convenient exports
pub mod models {
    pub use super::{Arg, Instance, Template};
    pub mod tag {
        pub use super::super::{Tag as Template, TagItem as Item};
    }
    pub mod amount {
        pub use super::super::{Amount as Template, AmountItem as Item};
    }
}

/// Represents parameters to a template expansion
#[derive(Debug)]
pub struct Instance<'i> {
    /// name of the template
    label: &'i str,
    /// positional arguments
    positional: Vec<Arg<'i>>,
    /// named arguments
    named: Vec<(&'i str, Arg<'i>)>,
    /// reference to the source file
    loc: error::Loc<'i>,
}

/// A single argument to a template or instanciation
#[derive(Debug, Clone, Copy)]
pub enum Arg<'i> {
    Amount(fields::Amount),
    Tag(&'i str),
}

/// A description of a template
#[derive(Debug)]
pub struct Template<'i> {
    /// positional arguments
    positional: Vec<&'i str>,
    /// named/optional arguments
    named: Vec<(&'i str, Arg<'i>)>,
    /// expands to a value field
    value: Amount<'i>,
    /// category field
    cat: Category,
    /// span field
    span: Span,
    /// expands to a tag field
    tag: Tag<'i>,
    /// reference to the source file
    loc: error::Loc<'i>,
}

/// Describes a field that expands to a tag
#[derive(Debug)]
pub struct Tag<'i>(Vec<TagItem<'i>>);

/// Possible contents of a tag field expansion
#[derive(Debug)]
pub enum TagItem<'i> {
    /// current day number
    Day,
    /// current month name
    Month,
    /// current year name
    Year,
    /// YYYY-Mmm-DD
    Date,
    /// name of day of week
    Weekday,
    /// a string literal
    Raw(&'i str),
    /// the name of an argument
    Arg(&'i str),
}

/// Describes a field that expands to an amount
#[derive(Debug)]
pub struct Amount<'i> {
    /// if `false` take the opposite
    sign: bool,
    /// perform summation of all contained values
    sum: Vec<AmountItem<'i>>,
}

/// Possible contents of an amount field expansion
#[derive(Debug)]
pub enum AmountItem<'i> {
    /// a numeric constant
    Cst(fields::Amount),
    /// the name of an argument
    Arg(&'i str),
}

impl<'i> Instance<'i> {
    pub fn new(
        label: &'i str,
        positional: Vec<Arg<'i>>,
        named: Vec<(&'i str, Arg<'i>)>,
        loc: error::Loc<'i>,
    ) -> Self {
        Self {
            label,
            positional,
            named,
            loc,
        }
    }
}

impl<'i> Template<'i> {
    pub fn new(
        positional: Vec<&'i str>,
        named: Vec<(&'i str, Arg<'i>)>,
        value: Amount<'i>,
        cat: Category,
        span: Span,
        tag: Tag<'i>,
        loc: error::Loc<'i>,
    ) -> Self {
        Self {
            positional,
            named,
            value,
            cat,
            span,
            tag,
            loc,
        }
    }
}

impl<'i> Tag<'i> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Add a new item to the tag concatenation
    pub fn push(&mut self, item: TagItem<'i>) {
        self.0.push(item);
    }
}

impl<'i> Amount<'i> {
    pub fn new(sign: bool) -> Self {
        Self {
            sign,
            sum: Vec::new(),
        }
    }

    /// Add a new item to the amount summation
    pub fn push(&mut self, item: AmountItem<'i>) {
        self.sum.push(item);
    }
}

/// Entries are kept, templates are filtered out, instanciations are expanded
///
/// Template expansion may fail without it being indicated in the returned value
/// Caller should query `errs` to find out if all instances were correctly expanded
/// (e.g. with `errs.is_fatal()` or `errs.count_errors()`)
pub fn instanciate(errs: &mut error::Record, items: ast::Ast<'_>) -> Vec<Entry> {
    let mut entries = Vec::new();
    let mut templates = HashMap::new();
    use ast::*;
    'ast: for item in items {
        match item {
            Item::Entry(entry) => entries.push(entry),
            Item::Template(name, body) => {
                templates.insert(name.to_string(), body);
            }
            Item::Instance(date, instance) => {
                match instanciate_item(errs, instance, date, &templates) {
                    Some(inst) => entries.push(inst),
                    None => continue 'ast,
                }
            }
        }
    }
    entries
}

/// Attempts template expansion
///
/// - find a template with the correct name
/// - build a `HashMap` for arguments
/// - perform string concatenation
/// - check correct typing of val contents
/// - perform summation of value
fn instanciate_item(
    errs: &mut error::Record,
    instance: Instance<'_>,
    date: Date,
    templates: &HashMap<String, Template>,
) -> Option<Entry> {
    let templ = match templates.get(instance.label) {
        None => {
            errs.make("Undeclared template")
                .span(
                    &instance.loc,
                    format!("attempt to instanciate {}", instance.label),
                )
                .text(format!("'{}' is not declared", instance.label))
                .hint("Maybe a typo ?");
            return None;
        }
        Some(t) => t,
    };
    let args = build_arguments(errs, &instance, templ)?;
    perform_replacements(errs, &instance, templ, args, date)
}

/// Construct `HashMap` of arguments
///
/// - check that lists of positional arguments are of matching length
/// - zip them together
/// - insert default values for named arguments
/// - overwrite with provided values
fn build_arguments<'i>(
    errs: &mut error::Record,
    inst: &Instance<'i>,
    templ: &Template<'i>,
) -> Option<HashMap<String, Arg<'i>>> {
    // check number of positional arguments
    let len_inst = inst.positional.len();
    let len_templ = templ.positional.len();
    if len_inst != len_templ {
        errs.make("Argcount mismatch")
            .span(
                &inst.loc,
                format!("instanciation provides {} arguments", len_inst),
            )
            .span(
                &templ.loc,
                format!("template expects {} arguments", len_templ),
            )
            .text("Fix the count mismatch")
            .hint(if len_inst > len_templ {
                format!(
                    "remove {} arguments from instanciation",
                    len_inst - len_templ
                )
            } else {
                format!("provide the {} missing arguments", len_templ - len_inst)
            });
        return None;
    }
    let mut args = HashMap::new();
    for (name, val) in templ.positional.iter().zip(inst.positional.iter()) {
        args.insert(name.to_string(), *val);
    }
    // template first so that instance overrides them
    for (name, val) in templ.named.iter() {
        args.insert(name.to_string(), *val);
    }
    for (name, val) in inst.named.iter() {
        args.insert(name.to_string(), *val);
    }
    Some(args)
}

/// Expand amount and tag
///
/// Also checks for unused arguments and needless typing constraints
fn perform_replacements(
    errs: &mut error::Record,
    inst: &Instance,
    templ: &Template,
    args: HashMap<String, Arg>,
    date: Date,
) -> Option<Entry> {
    let (value, used_val) = instantiate_amount(errs, inst, templ, &args)?;
    let (tag, used_tag) = instanciate_tag(errs, inst, templ, &args, date)?;
    for (argname, argval) in args.iter() {
        let use_v = used_val.contains(argname);
        let use_t = used_tag.contains(argname);
        match (argval, use_v, use_t) {
            (_, false, false) => {
                errs.make("Unused argument")
                    .nonfatal()
                    .span(&inst.loc, format!("in instanciation of '{}'", inst.label))
                    .text(format!("Argument '{}' is provided but not used", argname))
                    .span(&templ.loc, "defined here")
                    .hint("remove argument or use in template");
            }
            (Arg::Amount(a), false, true) => {
                errs.make("Needless amount")
                    .nonfatal()
                    .span(&inst.loc, format!("in instanciation of '{}'", inst.label))
                    .text(format!(
                        "Argument '{}' has type amount but could be a string",
                        argname
                    ))
                    .span(&templ.loc, "defined here")
                    .hint("argument is used only in tag field")
                    .hint(format!("change to string '\"{}\"' or use in val field", a));
            }
            _ => (),
        }
    }
    Some(Entry::from(date, value, templ.cat, templ.span, tag))
}

/// Expand amount
///
/// - handle missing arguments
/// - type checking of string arguments that can't be converted to values
/// - calculate sum of result
/// - negate if `!templ.sign`
///
/// Returns the final amount and a `HashSet` of used arguments
fn instantiate_amount(
    errs: &mut error::Record,
    inst: &Instance,
    templ: &Template,
    args: &HashMap<String, Arg>,
) -> Option<(fields::Amount, HashSet<String>)> {
    let mut sum = fields::Amount::zero();
    let mut used = HashSet::new();
    for item in &templ.value.sum {
        match item {
            &AmountItem::Cst(n) => sum += n,
            AmountItem::Arg(a) => {
                used.insert(a.to_string());
                match args.get(*a) {
                    None => {
                        errs.make("Missing argument")
                            .span(&inst.loc, format!("in instanciation of '{}'", inst.label))
                            .text(format!("Argument '{}' is not provided", a))
                            .span(&templ.loc, "defined here")
                            .hint("remove argument from template body")
                            .hint(format!("or provide a default value: '{}=0'", a));
                        return None;
                    }
                    Some(&Arg::Amount(n)) => sum += n,
                    Some(Arg::Tag(_)) => {
                        errs.make("Type mismatch")
                            .span(&inst.loc, format!("in instanciation of '{}'", inst.label))
                            .text("Cannot treat tag as a monetary value")
                            .span(&templ.loc, "defined here")
                            .hint("make it a value")
                            .hint("or remove from amount calculation");
                        return None;
                    }
                }
            }
        }
    }
    Some((if templ.value.sign { sum } else { -sum }, used))
}

/// Expand tag
///
/// - read date if required in concatenation
/// - handle missing arguments
/// - concatenate all into a single string
///
/// Returns the final tag and a `HashSet` of used arguments
fn instanciate_tag(
    errs: &mut error::Record,
    inst: &Instance,
    templ: &Template,
    args: &HashMap<String, Arg>,
    date: Date,
) -> Option<(fields::Tag, HashSet<String>)> {
    let mut tag = String::new();
    let mut used = HashSet::new();
    for item in &templ.tag.0 {
        match item {
            TagItem::Day => tag.push_str(&date.day().to_string()),
            TagItem::Month => tag.push_str(&date.month().to_string()),
            TagItem::Year => tag.push_str(&date.year().to_string()),
            TagItem::Date => tag.push_str(&date.to_string()),
            TagItem::Weekday => tag.push_str(&date.weekday().to_string()),
            TagItem::Raw(s) => tag.push_str(s),
            TagItem::Arg(a) => {
                used.insert(a.to_string());
                match args.get(*a) {
                    None => {
                        errs.make("Missing argument")
                            .span(&inst.loc, format!("in instanciation of '{}'", inst.label))
                            .text(format!("Argument '{}' is not provided", a))
                            .span(&templ.loc, "defined here")
                            .hint("remove argument from template body")
                            .hint(format!("or provide a default value: '{}=0'", a));
                        return None;
                    }
                    Some(Arg::Amount(amount)) => tag.push_str(&amount.to_string()),
                    Some(Arg::Tag(t)) => tag.push_str(t),
                }
            }
        }
    }
    Some((fields::Tag(tag), used))
}
