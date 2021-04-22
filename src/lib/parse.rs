#![allow(clippy::upper_case_acronyms)]

use pest_derive::*;
use pest::{
    Parser,
    iterators::{Pair, Pairs},
};

use crate::lib::{
    entry::{self, Entry, Amount, Tag, Span, Category},
    template::{self, Arg, Instance, models::*},
    date::{Date, Month},
    error::{ErrorRecord, Error, Loc}
};

pub mod ast {
    pub use super::{
        Ast,
        AstItem,
    };
}

#[derive(Parser)]
#[grammar = "billig.pest"]
pub struct BilligParser;

pub type Ast<'i> = Vec<AstItem<'i>>;

#[derive(Debug)]
pub enum AstItem<'i> {
    Entry(Date, Entry),
    Instance(Date, Loc<'i>, Instance<'i>),
    Template(&'i str, Loc<'i>, Template<'i>),
}

pub fn extract<'i>(path: &'i str, errs: &mut ErrorRecord, contents: &'i str) -> Ast<'i> {
    let contents = match BilligParser::parse(Rule::program, contents) {
        Ok(contents) => contents,
        Err(e) => {
            Error::new("Parsing failure")
                .with_error(e.with_path(path))
                .register(errs);
            return Vec::new();
        }
    };
    validate(path, errs, contents)
}

// extract contents of wrapper rule
macro_rules! subrule {
    ( $node:expr, $rule:expr ) => {{
        let node = $node;
        assert_eq!(node.as_rule(), $rule);
        let mut items = node.into_inner().into_iter();
        let fst = items
            .next()
            .unwrap_or_else(|| panic!("{:?} has no subrule", $rule));
        if items.next().is_some() {
            panic!("{:?} has several subrules", $rule);
        }
        fst
    }};
    ( $node:expr ) => {{
        let mut items = $node.into_inner().into_iter();
        let fst = items.next().unwrap_or_else(|| panic!("No subrule"));
        if items.next().is_some() {
            panic!("Several subrules");
        }
        fst
    }};
}

// get first and rest of inner
macro_rules! decapitate {
    ( $node:expr ) => {{
        let mut items = $node.into_inner().into_iter();
        let fst = items.next().unwrap_or_else(|| panic!("No head"));
        (fst, items)
    }};
}

// extract two-element inner
macro_rules! pair {
    ( $node:expr ) => {{
        let mut items = $node.into_inner().into_iter();
        let fst = items.next().unwrap_or_else(|| panic!("No 1st"));
        let snd = items.next().unwrap_or_else(|| panic!("No 2nd"));
        assert!(items.next().is_none());
        (fst, snd)
    }};
}

// extract three-element inner
macro_rules! triplet {
    ( $node:expr ) => {{
        let mut items = $node.into_inner().into_iter();
        let fst = items.next().unwrap_or_else(|| panic!("No 1st"));
        let snd = items.next().unwrap_or_else(|| panic!("No 2nd"));
        let thr = items.next().unwrap_or_else(|| panic!("No 3rd"));
        assert!(items.next().is_none());
        (fst, snd, thr)
    }};
}

// pair to usize contents
macro_rules! parse_usize {
    ( $node:expr ) => {
        $node.as_str().parse::<usize>().unwrap()
    };
}

// pair to amount contents
macro_rules! parse_amount {
    ( $node:expr ) => {
        ($node.as_str().parse::<f64>().unwrap() * 100.0).round() as isize
    };
}

// set-once value
macro_rules! set_or_fail {
    ( $errs:expr, $var:expr, $val:expr, $name:expr, $loc:expr) => {{
        if $var.is_some() {
            Error::new("Duplicate field definition")
                .with_span(&$loc, format!("attempt to override {}", $name))
                .with_text("Each field may only be defined once")
                .with_hint("remove one of the field definitions")
                .register($errs);
            return None;
        }
        $var = Some($val);
    }};
}

// non-optional value
macro_rules! unwrap_or_fail {
    ( $errs:expr, $val:expr, $name:expr, $loc:expr ) => {{
        match $val {
            Some(v) => v,
            None => {
                let name = $name;
                let hint_value = match name {
                    "tag" => "\"Some information\"",
                    "val" => "42.0",
                    "span" => "Year<Succ> 1",
                    "type" => "Food",
                    _ => unreachable!(),
                };
                Error::new("Missing field definition")
                    .with_span(&$loc, format!("'{}' may not be omitted", name))
                    .with_text("Each field must be defined once")
                    .with_hint(format!("add definition for the missing field: '{} {}'", name, hint_value))
                    .register($errs);
                return None;
            }
        }
    }};
}

pub fn validate<'i>(path: &'i str, errs: &mut ErrorRecord, pairs: Pairs<'i, Rule>) -> Ast<'i> {
    let mut ast = Vec::new();
    'pairs: for pair in pairs {
        match pair.as_rule() {
            Rule::item => {
                for item in pair.into_inner() {
                    let loc = (path, item.as_span().clone());
                    match item.as_rule() {
                        Rule::template_descriptor => {
                            let (name, templ) = match validate_template(path, errs, item) {
                                Some(x) => x,
                                None => continue 'pairs,
                            };
                            ast.push(AstItem::Template(name, loc, templ));
                        }
                        Rule::entries_year => {
                            let (head, body) = decapitate!(item);
                            assert_eq!(head.as_rule(), Rule::marker_year);
                            let year = parse_usize!(head);
                            let items = validate_year(path, errs, year, body.collect::<Vec<_>>());
                            for item in items {
                                ast.push(item);
                            }
                        }
                        _ => unreachable!(),
                    }
                }
            }
            Rule::EOI => break,
            _ => unreachable!(),
        }
    }
    ast
}

fn validate_template<'i>(path: &'i str, errs: &mut ErrorRecord, pair: Pair<'i, Rule>) -> Option<(&'i str, Template<'i>)> {
    let loc = (path, pair.as_span().clone());
    let (id, args, body) = triplet!(pair);
    assert_eq!(id.as_rule(), Rule::identifier);
    let identifier = id.as_str();
    assert_eq!(args.as_rule(), Rule::template_args);
    let (positional, named) = read_args(args.into_inner());
    assert_eq!(body.as_rule(), Rule::template_expansion_contents);
    let mut value: Option<AmountTemplate> = None;
    let mut cat: Option<Category> = None;
    let mut span: Option<Span> = None;
    let mut tag: Option<TagTemplate> = None;
    for sub in body.into_inner() {
        match sub.as_rule() {
            Rule::template_val => {
                set_or_fail!(
                    errs,
                    value,
                    read_template_amount(subrule!(subrule!(sub), Rule::template_money_amount)),
                    "val",
                    loc
                );
            }
            Rule::entry_type => {
                set_or_fail!(errs, cat, read_cat(subrule!(sub)), "type", loc);
            }
            Rule::entry_span => {
                set_or_fail!(errs, span, read_span(subrule!(sub)), "span", loc);
            }
            Rule::template_tag => {
                set_or_fail!(errs, tag, read_template_tag(subrule!(sub)), "tag", loc);
            }
            _ => unreachable!(),
        }
    }
    let value = unwrap_or_fail!(errs, value, "val", loc);
    let cat = unwrap_or_fail!(errs, cat, "cat", loc);
    let span = unwrap_or_fail!(errs, span, "span", loc);
    let tag = unwrap_or_fail!(errs, tag, "tag", loc);
    Some((
        identifier,
        Template {
            positional,
            named,
            value,
            cat,
            span,
            tag,
        },
    ))
}

fn read_args(pairs: Pairs<'_, Rule>) -> (Vec<&str>, Vec<(&str, Arg)>) {
    let mut positional = Vec::new();
    let mut named = Vec::new();
    for pair in pairs {
        match read_arg(pair) {
            (arg, None) => positional.push(arg),
            (arg, Some(deflt)) => named.push((arg, deflt)),
        }
    }
    (positional, named)
}

fn read_arg(pair: Pair<'_, Rule>) -> (&str, Option<Arg>) {
    match pair.as_rule() {
        Rule::template_positional_arg => {
            let name = pair.as_str();
            (name, None)
        }
        Rule::template_named_arg => {
            let (name, default) = pair!(pair);
            let name = name.as_str();
            let default = {
                match default.as_rule() {
                    Rule::money_amount => Arg::Amount(read_amount(default)),
                    Rule::tag_text => {
                        Arg::Tag(subrule!(default, Rule::tag_text).as_str())
                    }
                    _ => {
                        unreachable!()
                    }
                }
            };
            (name, Some(default))
        }
        _ => unreachable!(),
    }
}

fn read_amount(item: Pair<'_, Rule>) -> Amount {
    assert_eq!(item.as_rule(), Rule::money_amount);
    Amount(parse_amount!(item))
}

fn read_template_amount(pair: Pair<'_, Rule>) -> AmountTemplate {
    let (sign, pair) = match pair.as_rule() {
        Rule::builtin_neg => (false, subrule!(pair)),
        _ => (true, pair),
    };
    let items = match pair.as_rule() {
        Rule::builtin_sum => subrule!(pair)
            .into_inner()
            .into_iter()
            .map(|it| subrule!(it))
            .collect::<Vec<_>>(),
        _ => vec![pair],
    };
    let mut sum = Vec::new();
    for item in items {
        match item.as_rule() {
            Rule::money_amount => {
                sum.push(AmountTemplateItem::Cst(read_amount(item)));
            }
            Rule::template_arg_expand => {
                sum.push(AmountTemplateItem::Arg(subrule!(item).as_str()))
            }
            _ => unreachable!(),
        }
    }
    AmountTemplate { sign, sum }
}

fn read_cat(pair: Pair<'_, Rule>) -> Category {
    use entry::Category::*;
    match pair.as_str() {
        "Pay" => Salary,
        "Food" => Food,
        "Com" => Communication,
        "Mov" => Movement,
        "Pro" => School,
        "Clean" => Cleaning,
        "Home" => Home,
        _ => unreachable!(),
    }
}

fn read_span(pair: Pair<'_, Rule>) -> Span {
    let mut pair = pair.into_inner().into_iter().peekable();
    use entry::Duration::*;
    let duration = match pair.next().unwrap().as_str() {
        "Day" => Day,
        "Week" => Week,
        "Month" => Month,
        "Year" => Year,
        _ => unreachable!(),
    };
    use entry::Window::*;
    let window = pair
        .peek()
        .map(|it| {
            if it.as_rule() == Rule::span_window {
                Some(match it.as_str() {
                    "Curr" => Current,
                    "Post" => Posterior,
                    "Ante" => Anterior,
                    "Pred" => Precedent,
                    "Succ" => Successor,
                    _ => unreachable!(),
                })
            } else {
                None
            }
        })
        .flatten();
    if window.is_some() {
        pair.next();
    }
    let count = pair.next().map(|it| parse_usize!(it)).unwrap_or(1);
    Span {
        duration,
        window: window.unwrap_or(Current),
        count,
    }
}

fn read_template_tag(pair: Pair<'_, Rule>) -> TagTemplate {
    let concat = match pair.as_rule() {
        Rule::builtin_concat => subrule!(pair)
            .into_inner()
            .into_iter()
            .map(|it| subrule!(it, Rule::template_string))
            .collect::<Vec<_>>(),
        Rule::tag_text => vec![pair],
        _ => pair.into_inner().into_iter().collect::<Vec<_>>(),
    };
    let mut strs = Vec::new();
    use template::TagTemplateItem::*;
    for item in concat {
        strs.push(match item.as_rule() {
            Rule::tag_text => Raw(subrule!(item).as_str()),
            Rule::template_arg_expand => Arg(subrule!(item).as_str()),
            Rule::template_time => match item.as_str() {
                "@Day" => Day,
                "@Month" => Month,
                "@Year" => Year,
                "@Date" => Date,
                "@Weekday" => Weekday,
                _ => unreachable!(),
            },
            _ => unreachable!(),
        });
    }
    TagTemplate(strs)
}

fn validate_year<'i>(path: &'i str, errs: &mut ErrorRecord, year: usize, pairs: Vec<Pair<'i, Rule>>) -> Vec<AstItem<'i>> {
    let mut v = Vec::new();
    for pair in pairs {
        assert_eq!(pair.as_rule(), Rule::entries_month);
        let (month, rest) = decapitate!(pair);
        let month = Month::from(month.as_str());
        let items = validate_month(path, errs, year, month, rest.collect::<Vec<_>>());
        for item in items {
            v.push(item);
        }
    }
    v
}

fn validate_month<'i>(path: &'i str, errs: &mut ErrorRecord, year: usize, month: Month, pairs: Vec<Pair<'i, Rule>>) -> Vec<AstItem<'i>> {
    let mut v = Vec::new();
    'pairs: for pair in pairs {
        assert_eq!(pair.as_rule(), Rule::entries_day);
        let (day, rest) = decapitate!(pair);
        let loc = (path, day.as_span().clone());
        let day = parse_usize!(day);
        match Date::from(year, month, day) {
            Ok(date) => {
                let items = validate_day(path, errs, date, rest.collect::<Vec<_>>());
                for item in items {
                    v.push(item);
                }
            }
            Err(e) => {
                Error::new("Invalid date")
                    .with_span(&loc, "defined here")
                    .with_text(format!("{}", e))
                    .with_hint("choose a date that exists")
                    .register(errs);
                continue 'pairs;
            }
        }
    }
    v
}

fn validate_day<'i>(path: &'i str, errs: &mut ErrorRecord, date: Date, pairs: Vec<Pair<'i, Rule>>) -> Vec<AstItem<'i>> {
    let mut v = Vec::new();
    'pairs: for pair in pairs {
        let entry = subrule!(pair, Rule::entry);
        let loc = (path, entry.as_span().clone());
        match entry.as_rule() {
            Rule::expand_entry => {
                let res = read_expand_entry(entry);
                v.push(AstItem::Instance(date, loc, res));
            }
            Rule::plain_entry => {
                let res = match validate_plain_entry(path, errs, entry) {
                    Some(x) => x,
                    None => continue 'pairs,
                };
                v.push(AstItem::Entry(date, res));
            }
            _ => unreachable!(),
        }
    }
    v
}

fn read_expand_entry(pairs: Pair<'_, Rule>) -> Instance {
    let (label, args) = pair!(pairs);
    let label = label.as_str();
    let mut pos = Vec::new();
    let mut named = Vec::new();
    for arg in args.into_inner() {
        match arg.as_rule() {
            Rule::positional_arg => {
                pos.push(read_value(subrule!(arg)));
            }
            Rule::named_arg => {
                let (name, value) = pair!(arg);
                let name = name.as_str();
                let value = read_value(subrule!(value));
                named.push((name, value));
            }
            _ => unreachable!(),
        }
    }
    Instance { label, pos, named }
}

fn read_value(pair: Pair<'_, Rule>) -> Arg {
    match pair.as_rule() {
        Rule::money_amount => Arg::Amount(read_amount(pair)),
        Rule::tag_text => Arg::Tag(subrule!(pair).as_str()),
        _ => {
            unreachable!()
        }
    }
}

fn validate_plain_entry(path: &str, errs: &mut ErrorRecord, pair: Pair<'_, Rule>) -> Option<Entry> {
    let loc = (path, pair.as_span().clone());
    let mut value: Option<Amount> = None;
    let mut cat: Option<Category> = None;
    let mut span: Option<Span> = None;
    let mut tag: Option<Tag> = None;
    for item in pair.into_inner() {
        match item.as_rule() {
            Rule::entry_val => {
                set_or_fail!(errs, value, Amount(parse_amount!(subrule!(item))), "val", loc);
            }
            Rule::entry_type => {
                set_or_fail!(errs, cat, read_cat(subrule!(item)), "cat", loc);
            }
            Rule::entry_span => {
                set_or_fail!(errs, span, read_span(subrule!(item)), "span", loc);
            }
            Rule::entry_tag => {
                set_or_fail!(
                    errs,
                    tag,
                    Tag(subrule!(item).into_inner().as_str().to_string()),
                    "tag",
                    loc
                );
            }
            _ => unreachable!(),
        }
    }
    let value = unwrap_or_fail!(errs, value, "val", loc);
    let cat = unwrap_or_fail!(errs, cat, "cat", loc);
    let span = unwrap_or_fail!(errs, span, "span", loc);
    let tag = unwrap_or_fail!(errs, tag, "tag", loc);
    Some(Entry {
        value,
        cat,
        span,
        tag,
    })
}
