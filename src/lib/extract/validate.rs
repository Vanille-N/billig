
use crate::lib::extract::{
    entry::{Category, Duration, Entry, Span, Window},
    instance::{Arg, Instance},
    parse::Rule,
    template::{AmountTemplate, AmountTemplateItem, TagTemplate, TagTemplateItem, Template},
    Amount, Tag,
};

use crate::lib::date::{Date, Month};

#[derive(Debug)]
pub enum AstItem<'i> {
    Entry(Date, Entry),
    Instance(Date, pest::Span<'i>, Instance),
    Template(String, pest::Span<'i>, Template),
}
pub type Ast<'i> = Vec<AstItem<'i>>;

use pest::{
    error::{Error, ErrorVariant},
    iterators::{Pair, Pairs},
};
pub type Result<T> = std::result::Result<T, Error<Rule>>;

// build and return pest::Error from message and span
macro_rules! failure {
    ( $msg:expr, $span:expr ) => {{
        let err: Error<Rule> = Error::new_from_span(
            ErrorVariant::CustomError {
                message: $msg.to_string(),
            },
            $span,
        );
        return Err(err);
    }};
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

// convert to string
macro_rules! as_string {
    ( $node:expr ) => {
        $node.as_str().to_string()
    };
}

// set-once value
macro_rules! set_or_fail {
    ( $var:expr, $val:expr, $name:expr, $loc:expr ) => {{
        if $var.is_some() {
            failure!(format!("Attempt to override {}", $name), $loc)
        }
        $var = Some($val);
    }};
}

// non-optional value
macro_rules! unwrap_or_fail {
    ( $val:expr, $name:expr, $loc:expr ) => {{
        match $val {
            Some(v) => v,
            None => failure!(format!("Unspecified {}", $name), $loc),
        }
    }};
}

pub fn validate(pairs: Pairs<'_, Rule>) -> Result<Ast> {
    let mut ast = Vec::new();
    for pair in pairs {
        match pair.as_rule() {
            Rule::item => {
                for item in pair.into_inner() {
                    let loc = item.as_span().clone();
                    match item.as_rule() {
                        Rule::comment => (),
                        Rule::template_descriptor => {
                            let (name, templ) = validate_template(item)?;
                            ast.push(AstItem::Template(name, loc, templ));
                        }
                        Rule::entries_year => {
                            let (head, body) = decapitate!(item);
                            assert_eq!(head.as_rule(), Rule::marker_year);
                            let year = parse_usize!(head);
                            let items = validate_year(year, body.collect::<Vec<_>>())?;
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
    Ok(ast)
}

fn validate_template(pair: Pair<'_, Rule>) -> Result<(String, Template)> {
    let loc = pair.as_span().clone();
    let (id, args, body) = triplet!(pair);
    assert_eq!(id.as_rule(), Rule::identifier);
    let identifier = as_string!(id);
    assert_eq!(args.as_rule(), Rule::template_args);
    let arguments = validate_args(args.into_inner())?;
    assert_eq!(body.as_rule(), Rule::template_expansion_contents);
    let mut value: Option<AmountTemplate> = None;
    let mut cat: Option<Category> = None;
    let mut span: Option<Span> = None;
    let mut tag: Option<TagTemplate> = None;
    for sub in body.into_inner() {
        match sub.as_rule() {
            Rule::template_val => {
                set_or_fail!(
                    value,
                    validate_template_amount(subrule!(subrule!(sub), Rule::template_money_amount))?,
                    "val",
                    loc
                );
            }
            Rule::entry_type => {
                set_or_fail!(cat, validate_cat(subrule!(sub))?, "type", loc);
            }
            Rule::entry_span => {
                set_or_fail!(span, validate_span(subrule!(sub))?, "span", loc);
            }
            Rule::template_tag => {
                set_or_fail!(tag, validate_template_tag(subrule!(sub))?, "tag", loc);
            }
            _ => unreachable!(),
        }
    }
    let value = unwrap_or_fail!(value, "val", loc);
    let cat = unwrap_or_fail!(cat, "cat", loc);
    let span = unwrap_or_fail!(span, "span", loc);
    let tag = unwrap_or_fail!(tag, "tag", loc);
    Ok((
        identifier,
        Template {
            arguments,
            value,
            cat,
            span,
            tag,
        },
    ))
}

fn validate_args(pairs: Pairs<'_, Rule>) -> Result<Vec<(String, Option<Arg>)>> {
    let mut args = Vec::new();
    for pair in pairs {
        args.push(validate_arg(pair)?);
    }
    Ok(args)
}

fn validate_arg(pair: Pair<'_, Rule>) -> Result<(String, Option<Arg>)> {
    match pair.as_rule() {
        Rule::template_positional_arg => {
            let name = as_string!(pair);
            Ok((name, None))
        }
        Rule::template_named_arg => {
            let (name, default) = pair!(pair);
            let name = as_string!(name);
            let default = {
                match default.as_rule() {
                    Rule::money_amount => Arg::Amount(validate_amount(default)?),
                    Rule::tag_text => {
                        let tag = subrule!(default, Rule::tag_text);
                        Arg::Tag(Tag(as_string!(tag)))
                    }
                    _ => {
                        unreachable!()
                    }
                }
            };
            Ok((name, Some(default)))
        }
        _ => unreachable!(),
    }
}

fn validate_amount(item: Pair<'_, Rule>) -> Result<Amount> {
    assert_eq!(item.as_rule(), Rule::money_amount);
    Ok(Amount(parse_amount!(item)))
}

fn validate_template_amount(pair: Pair<'_, Rule>) -> Result<AmountTemplate> {
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
                sum.push(AmountTemplateItem::Cst(validate_amount(item)?));
            }
            Rule::template_arg_expand => {
                sum.push(AmountTemplateItem::Arg(as_string!(subrule!(item))))
            }
            _ => unreachable!(),
        }
    }
    Ok(AmountTemplate { sign, sum })
}

fn validate_cat(pair: Pair<'_, Rule>) -> Result<Category> {
    use Category::*;
    Ok(match pair.as_str() {
        "Pay" => Salary,
        "Food" => Food,
        "Com" => Communication,
        "Mov" => Movement,
        "Scol" => School,
        "Clean" => Cleaning,
        "Home" => Home,
        _ => unreachable!(),
    })
}

fn validate_span(pair: Pair<'_, Rule>) -> Result<Span> {
    let mut pair = pair.into_inner().into_iter().peekable();
    use Duration::*;
    let duration = match pair.next().unwrap().as_str() {
        "Day" => Day,
        "Week" => Week,
        "Month" => Month,
        "Year" => Year,
        _ => unreachable!(),
    };
    use Window::*;
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
    Ok(Span {
        duration,
        window: window.unwrap_or(Current),
        count,
    })
}

fn validate_template_tag(pair: Pair<'_, Rule>) -> Result<TagTemplate> {
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
    use TagTemplateItem::*;
    for item in concat {
        strs.push(match item.as_rule() {
            Rule::tag_text => Raw(Tag(as_string!(subrule!(item)))),
            Rule::template_arg_expand => Arg(as_string!(subrule!(item))),
            Rule::template_time => match item.as_str() {
                "@Day" => Day,
                "@Month" => Month,
                "@Year" => Year,
                _ => unreachable!(),
            },
            _ => unreachable!(),
        });
    }
    Ok(TagTemplate(strs))
}

fn validate_year(year: usize, pairs: Vec<Pair<'_, Rule>>) -> Result<Vec<AstItem>> {
    let mut v = Vec::new();
    for pair in pairs {
        assert_eq!(pair.as_rule(), Rule::entries_month);
        let (month, rest) = decapitate!(pair);
        let month = Month::from(month.as_str());
        let items = validate_month(year, month, rest.collect::<Vec<_>>())?;
        for item in items {
            v.push(item);
        }
    }
    Ok(v)
}

fn validate_month(year: usize, month: Month, pairs: Vec<Pair<'_, Rule>>) -> Result<Vec<AstItem>> {
    let mut v = Vec::new();
    for pair in pairs {
        let loc = pair.as_span().clone();
        assert_eq!(pair.as_rule(), Rule::entries_day);
        let (day, rest) = decapitate!(pair);
        let day = parse_usize!(day);
        match Date::from(year, month, day) {
            Ok(date) => {
                let items = validate_day(date, rest.collect::<Vec<_>>())?;
                for item in items {
                    v.push(item);
                }
            }
            Err(e) => {
                let err: Error<Rule> = Error::new_from_span(
                    ErrorVariant::CustomError {
                        message: format!("{}", e),
                    },
                    loc,
                );
                return Err(err);
            }
        }
    }
    Ok(v)
}

fn validate_day(date: Date, pairs: Vec<Pair<'_, Rule>>) -> Result<Vec<AstItem>> {
    let mut v = Vec::new();
    for pair in pairs {
        let entry = subrule!(pair, Rule::entry);
        let loc = entry.as_span().clone();
        match entry.as_rule() {
            Rule::expand_entry => {
                let res = validate_expand_entry(entry)?;
                v.push(AstItem::Instance(date.clone(), loc, res));
            }
            Rule::plain_entry => {
                let res = validate_plain_entry(entry)?;
                v.push(AstItem::Entry(date.clone(), res));
            }
            _ => unreachable!(),
        }
    }
    Ok(v)
}

fn validate_expand_entry(pairs: Pair<'_, Rule>) -> Result<Instance> {
    let (label, args) = pair!(pairs);
    let label = as_string!(label);
    let mut pos = Vec::new();
    let mut named = Vec::new();
    for arg in args.into_inner() {
        match arg.as_rule() {
            Rule::positional_arg => {
                pos.push(validate_value(subrule!(arg)).unwrap());
            }
            Rule::named_arg => {
                let (name, value) = pair!(arg);
                let name = as_string!(name);
                let value = validate_value(subrule!(value)).unwrap();
                named.push((name, value));
            }
            _ => unreachable!(),
        }
    }
    Ok(Instance { label, pos, named })
}

fn validate_value(pair: Pair<'_, Rule>) -> Result<Arg> {
    Ok(match pair.as_rule() {
        Rule::money_amount => Arg::Amount(validate_amount(pair)?),
        Rule::tag_text => Arg::Tag(Tag(subrule!(pair).as_str().to_string())),
        _ => {
            unreachable!()
        }
    })
}

fn validate_plain_entry(pair: Pair<'_, Rule>) -> Result<Entry> {
    let loc = pair.as_span().clone();
    let mut value: Option<Amount> = None;
    let mut cat: Option<Category> = None;
    let mut span: Option<Span> = None;
    let mut tag: Option<Tag> = None;
    for item in pair.into_inner() {
        match item.as_rule() {
            Rule::entry_val => {
                set_or_fail!(value, Amount(parse_amount!(subrule!(item))), "val", loc);
            }
            Rule::entry_type => {
                set_or_fail!(cat, validate_cat(subrule!(item))?, "cat", loc);
            }
            Rule::entry_span => {
                set_or_fail!(span, validate_span(subrule!(item))?, "span", loc);
            }
            Rule::entry_tag => {
                set_or_fail!(
                    tag,
                    Tag(as_string!(subrule!(item).into_inner())),
                    "tag",
                    loc
                );
            }
            _ => unreachable!(),
        }
    }
    let value = unwrap_or_fail!(value, "val", loc);
    let cat = unwrap_or_fail!(cat, "cat", loc);
    let span = unwrap_or_fail!(span, "span", loc);
    let tag = unwrap_or_fail!(tag, "tag", loc);
    Ok(Entry {
        value,
        cat,
        span,
        tag,
    })
}
