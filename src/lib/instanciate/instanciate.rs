use crate::lib::date::Date;
use crate::lib::extract::{
    entry::Entry,
    instance::Instance,
    template::Template,
    validate::{Ast, AstItem, Result},
    parse::Rule,
};

use std::collections::HashMap;
use pest::error::{Error, ErrorVariant};

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
    unimplemented!()
}
