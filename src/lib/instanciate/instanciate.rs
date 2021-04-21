use crate::lib::date::Date;
use crate::lib::extract::{
    entry::Entry,
    instance::{Arg, Instance},
    parse::Rule,
    template::Template,
    validate::{Ast, AstItem, Result},
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
    let args = build_arguments(&instance, loc, templ)?;
    println!("{:?}", args);
    unimplemented!()
}

macro_rules! wrong_argcount {
    ( $name:expr, $len1:expr, $loc1:expr, $len2:expr, $loc2:expr ) => {{
        let len1 = $len1;
        let len2 = $len2;
        let name = $name;
        let def: Error<Rule> = Error::new_from_span(
            ErrorVariant::CustomError {
                message: format!("{} defined here", name),
            },
            $loc2,
        );
        let msg = format!(
            "In instanciation of {}\nWrong number of positional arguments\n expected {}, got {}\n{}",
            name, len2, len1, def,
        );   
        let err: Error<Rule> = Error::new_from_span(
            ErrorVariant::CustomError {
                message: msg,
            },
            $loc1,
        );
        return Err(err);
    }};
}

fn build_arguments<'i>(
    instance: &Instance<'i>,
    loc: pest::Span,
    template: &(pest::Span<'i>, Template<'i>),
) -> Result<HashMap<String, Arg<'i>>> {
    // check number of positional arguments
    if instance.pos.len() != template.1.positional.len() {
        wrong_argcount!(
            instance.label,
            instance.pos.len(),
            loc,
            template.1.positional.len(),
            template.0.clone()
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
