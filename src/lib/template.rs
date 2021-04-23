use std::collections::{HashMap, HashSet};

use crate::lib::{
    parse::ast::*,
    error::{Error, ErrorRecord, Loc},
    entry::{Entry, fields::*},
    date::Date,
};

pub mod models {
    pub use super::{
        Template,
        TagTemplate,
        AmountTemplate,
        AmountTemplateItem,
    };
}

#[derive(Debug)]
pub struct Instance<'i> {
    pub label: &'i str,
    pub pos: Vec<Arg<'i>>,
    pub named: Vec<(&'i str, Arg<'i>)>,
}

#[derive(Debug, Clone, Copy)]
pub enum Arg<'i> {
    Amount(Amount),
    Tag(&'i str),
}
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

pub fn instanciate(errs: &mut ErrorRecord, ast: Ast<'_>) -> Vec<(Date, Entry)> {
    let mut entries = Vec::new();
    let mut templates = HashMap::new();
    'ast: for item in ast {
        match item {
            AstItem::Entry(date, entry) => entries.push((date, entry)),
            AstItem::Template(name, loc, body) => {
                templates.insert(name.to_string(), (loc, body));
            }
            AstItem::Instance(date, loc, instance) => {
                match instanciate_item(errs, instance, date, &loc, &templates) {
                    Some(inst) => entries.push((date, inst)),
                    None => continue 'ast,
                }
            }
        }
    }
    entries
}

fn instanciate_item(
    errs: &mut ErrorRecord,
    instance: Instance<'_>,
    date: Date,
    loc: &Loc,
    templates: &HashMap<String, (Loc, Template)>,
) -> Option<Entry> {
    let templ = match templates.get(instance.label) {
        None => {
            Error::new("Undeclared template")
                .with_span(loc, format!("attempt to instanciate {}", instance.label))
                .with_text(format!("'{}' is not declared", instance.label))
                .with_hint("Maybe a typo ?")
                .register(errs);
            return None;
        }
        Some(t) => t,
    };
    let args = build_arguments(errs, &instance, loc, templ)?;
    perform_replacements(errs, &instance.label, loc, templ, args, date)
}

fn build_arguments<'i>(
    errs: &mut ErrorRecord,
    instance: &Instance<'i>,
    loc: &Loc,
    template: &(Loc<'i>, Template<'i>),
) -> Option<HashMap<String, Arg<'i>>> {
    // check number of positional arguments
    let len_inst = instance.pos.len();
    let len_templ = template.1.positional.len();
    if len_inst != len_templ {
        Error::new("Argcount mismatch")
            .with_span(loc, format!("instanciation provides {} arguments", instance.pos.len()))
            .with_span(&template.0, format!("template expects {} arguments", template.1.positional.len()))
            .with_text("Fix the count mismatch")
            .with_hint(if len_inst > len_templ {
                format!("remove {} arguments from instanciation", len_inst - len_templ)
            } else {
                format!("provide the {} missing arguments", len_templ - len_inst)
            })
            .register(errs);
        return None;
    }
    let mut args = HashMap::new();
    for (name, val) in template.1.positional.iter().zip(instance.pos.iter()) {
        args.insert(name.to_string(), *val);
    }
    // template first so that instance overrides them
    for (name, val) in template.1.named.iter() {
        args.insert(name.to_string(), *val);
    }
    for (name, val) in instance.named.iter() {
        args.insert(name.to_string(), *val);
    }
    Some(args)
}

fn perform_replacements(
    errs: &mut ErrorRecord,
    name: &str,
    loc: &Loc,
    templ: &(Loc, Template),
    args: HashMap<String, Arg>,
    date: Date,
) -> Option<Entry> {
    let (value, used_val) = instantiate_amount(errs, name, loc, &templ.0, &templ.1.value, &args)?;
    let (tag, used_tag) = instanciate_tag(errs, name, loc, &templ.0, &templ.1.tag, &args, date)?;
    for (argname, argval) in args.iter() {
        let use_v = used_val.contains(argname);
        let use_t = used_tag.contains(argname);
        match (argval, use_v, use_t) {
            (_, false, false) => {
                Error::new("Unused argument")
                    .nonfatal()
                    .with_span(&loc, format!("in instanciation of '{}'", name))
                    .with_text(format!("Argument {} is provided but not used", argname))
                    .with_span(&templ.0, "defined here")
                    .with_hint("remove argument or use in template")
                    .register(errs);
            }
            (Arg::Amount(a), false, true) => {
                Error::new("Needless amount")
                    .nonfatal()
                    .with_span(&loc, format!("in instanciation of '{}'", name))
                    .with_text(format!("Argument '{}' has type amount but could be a string", argname))
                    .with_span(&templ.0, "defined here")
                    .with_hint("argument is used only in tag field")
                    .with_hint(format!("change to string '\"{}\"' or use in val field", a))
                    .register(errs);
            }
            _ => (),
        }
    }
    Some(Entry {
        value,
        cat: templ.1.cat,
        span: templ.1.span,
        tag,
    })
}

fn instantiate_amount(
    errs: &mut ErrorRecord,
    name: &str,
    loc_inst: &Loc,
    loc_templ: &Loc,
    templ: &AmountTemplate,
    args: &HashMap<String, Arg>,
) -> Option<(Amount, HashSet<String>)> {
    let mut sum = 0;
    let mut used = HashSet::new();
    for item in &templ.sum {
        match item {
            AmountTemplateItem::Cst(Amount(n)) => sum += n,
            AmountTemplateItem::Arg(a) => {
                used.insert(a.to_string());
                match args.get(*a) {
                    None => {
                        Error::new("Missing argument")
                            .with_span(loc_inst, format!("in instanciation of '{}'", name))
                            .with_text(format!("Argument '{}' is not provided", a))
                            .with_span(loc_templ, "defined here")
                            .with_hint("remove argument from template body")
                            .with_hint(format!("or provide a default value: '{}=0'", a))
                            .register(errs);
                        return None;
                    }
                    Some(Arg::Amount(Amount(n))) => sum += n,
                    Some(Arg::Tag(_)) => {
                        Error::new("Type mismatch")
                            .with_span(loc_inst, format!("in instanciation of '{}'", name))
                            .with_text("Cannot treat tag as a monetary value")
                            .with_span(loc_templ, "defined here")
                            .with_hint("make it a value")
                            .with_hint("or remove from amount calculation")
                            .register(errs);
                        return None;
                    }
                }
            }
        }
    }
    Some((Amount(if templ.sign { sum } else { -sum }), used))
}

fn instanciate_tag(
    errs: &mut ErrorRecord,
    name: &str,
    loc_inst: &Loc,
    loc_templ: &Loc,
    templ: &TagTemplate,
    args: &HashMap<String, Arg>,
    date: Date,
) -> Option<(Tag, HashSet<String>)> {
    let mut tag = String::new();
    let mut used = HashSet::new();
    for item in &templ.0 {
        match item {
            TagTemplateItem::Day => tag.push_str(&date.day().to_string()),
            TagTemplateItem::Month => tag.push_str(&date.month().to_string()),
            TagTemplateItem::Year => tag.push_str(&date.year().to_string()),
            TagTemplateItem::Date => tag.push_str(&date.to_string()),
            TagTemplateItem::Weekday => tag.push_str(&date.weekday().to_string()),
            TagTemplateItem::Raw(s) => tag.push_str(s),
            TagTemplateItem::Arg(a) => {
                used.insert(a.to_string());
                match args.get(*a) {
                    None => {
                        Error::new("Missing argument")
                            .with_span(loc_inst, format!("in instanciation of '{}'", name))
                            .with_text(format!("Argument '{}' is not provided", a))
                            .with_span(loc_templ, "defined here")
                            .with_hint("remove argument from template body")
                            .with_hint(format!("or provide a default value: '{}=0'", a))
                            .register(errs);
                        return None;
                    }
                    Some(Arg::Amount(amount)) => tag.push_str(&amount.to_string()),
                    Some(Arg::Tag(t)) => tag.push_str(t),
                }
            }
        }
    }
    Some((Tag(tag), used))
}
