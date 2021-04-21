use crate::lib::date::Date;
use crate::lib::extract::{
    entry::Entry,
    instance::{Arg, Instance},
    parse::Rule,
    template::{AmountTemplate, AmountTemplateItem, TagTemplate, TagTemplateItem, Template},
    validate::{Ast, AstItem, Result},
    Amount, Tag,
};

use pest::error::{Error, ErrorVariant};
use std::collections::HashMap;

pub fn instanciate(ast: Ast<'_>) -> Result<Vec<(Date, Entry)>> {
    println!("{:?}", ast);
    let mut entries = Vec::new();
    let mut templates = HashMap::new();
    for item in ast {
        match item {
            AstItem::Entry(date, entry) => entries.push((date, entry)),
            AstItem::Template(name, loc, body) => {
                templates.insert(name.to_string(), (loc, body));
            }
            AstItem::Instance(date, loc, instance) => {
                let inst = instanciate_item(instance, date, loc, &templates)?;
                entries.push((date, inst));
            }
        }
    }
    println!("{:#?}", entries);
    panic!();
}

macro_rules! not_found {
    ( $loc:expr, $name:expr ) => {{
        let err: Error<Rule> = Error::new_from_span(
            ErrorVariant::CustomError {
                message: format!("'{}' is not a declared template", $name),
            },
            $loc,
        );
        return Err(err);
    }};
}

fn instanciate_item(
    instance: Instance<'_>,
    date: Date,
    loc: pest::Span,
    templates: &HashMap<String, (pest::Span, Template)>,
) -> Result<Entry> {
    let templ = match templates.get(instance.label) {
        None => not_found!(loc, instance.label),
        Some(t) => t,
    };
    let args = build_arguments(&instance, &loc, templ)?;
    perform_replacements(&instance.label, &loc, templ, args, date)
}

macro_rules! instanciation_failure {
    ( $name:expr, $loc_inst:expr, $loc_templ:expr, $msg:expr ) => {{
        let name = $name;
        let def: Error<Rule> = Error::new_from_span(
            ErrorVariant::CustomError {
                message: format!("{} defined here", name),
            },
            $loc_templ.clone(),
        );
        let msg = format!("In instanciation of {}\n{}\n{}", name, $msg, def,);
        let err: Error<Rule> = Error::new_from_span(
            ErrorVariant::CustomError { message: msg },
            $loc_inst.clone(),
        );
        return Err(err);
    }};
}

macro_rules! wrong_argcount {
    ( $name:expr, $len_inst:expr, $loc_inst:expr, $len_templ:expr, $loc_templ:expr ) => {{
        let len_inst = $len_inst;
        let len_templ = $len_templ;
        instanciation_failure!(
            $name,
            $loc_inst,
            $loc_templ,
            format!(
                "Wrong number of positional arguments\n expected {}, got {}",
                len_templ, len_inst,
            )
        );
    }};
}

fn build_arguments<'i>(
    instance: &Instance<'i>,
    loc: &pest::Span,
    template: &(pest::Span<'i>, Template<'i>),
) -> Result<HashMap<String, Arg<'i>>> {
    // check number of positional arguments
    if instance.pos.len() != template.1.positional.len() {
        wrong_argcount!(
            instance.label,
            instance.pos.len(),
            loc,
            template.1.positional.len(),
            template.0
        );
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
    Ok(args)
}

fn perform_replacements(
    name: &str,
    loc: &pest::Span,
    templ: &(pest::Span, Template),
    args: HashMap<String, Arg>,
    date: Date,
) -> Result<Entry> {
    let value = instantiate_amount(name, loc, &templ.0, &templ.1.value, &args)?;
    let tag = instanciate_tag(name, loc, &templ.0, &templ.1.tag, &args, date)?;
    Ok(Entry {
        value,
        cat: templ.1.cat,
        span: templ.1.span,
        tag,
    })
}

macro_rules! missing_argument {
    ( $name:expr, $loc_inst:expr, $loc_templ:expr, $argname:expr ) => {{
        instanciation_failure!(
            $name,
            $loc_inst,
            $loc_templ,
            format!("Argument {} was not provided", $argname)
        );
    }};
}

macro_rules! mismatched_type {
    ( $name:expr, $loc_inst:expr, $loc_templ:expr, $argname:expr ) => {{
        instanciation_failure!(
            $name,
            $loc_inst,
            $loc_templ,
            format!(
                "Cannot use '{}' of type label in amount summation",
                $argname
            )
        );
    }};
}

fn instantiate_amount(
    name: &str,
    loc_inst: &pest::Span,
    loc_templ: &pest::Span,
    templ: &AmountTemplate,
    args: &HashMap<String, Arg>,
) -> Result<Amount> {
    let mut sum = 0;
    for item in &templ.sum {
        match item {
            AmountTemplateItem::Cst(Amount(n)) => sum += n,
            AmountTemplateItem::Arg(a) => match args.get(*a) {
                None => missing_argument!(name, loc_inst, loc_templ, a),
                Some(Arg::Amount(Amount(n))) => sum += n,
                Some(Arg::Tag(_)) => mismatched_type!(name, loc_inst, loc_templ, name),
            },
        }
    }
    Ok(Amount(if templ.sign { sum } else { -sum }))
}

fn instanciate_tag(
    name: &str,
    loc_inst: &pest::Span,
    loc_templ: &pest::Span,
    templ: &TagTemplate,
    args: &HashMap<String, Arg>,
    date: Date,
) -> Result<Tag> {
    let mut tag = String::new();
    for item in &templ.0 {
        match item {
            TagTemplateItem::Day => tag.push_str(&date.day().to_string()),
            TagTemplateItem::Month => tag.push_str(&date.month().to_string()),
            TagTemplateItem::Year => tag.push_str(&date.year().to_string()),
            TagTemplateItem::Date => tag.push_str(&date.to_string()),
            TagTemplateItem::Weekday => tag.push_str(&date.weekday().to_string()),
            TagTemplateItem::Raw(s) => tag.push_str(s),
            TagTemplateItem::Arg(a) => match args.get(*a) {
                None => missing_argument!(name, loc_inst, loc_templ, a),
                Some(Arg::Amount(amount)) => tag.push_str(&amount.to_string()),
                Some(Arg::Tag(t)) => tag.push_str(t),
            },
        }
    }
    Ok(Tag(tag))
}
