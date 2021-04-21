pub type Ast = Vec<AstItem>;

use crate::lib::extract::{
    entry::{Category, Duration, Entry, Span, Window},
    instance::{Arg, Instance},
    parse::Rule,
    template::{AmountTemplate, AmountTemplateItem, TagTemplate, TagTemplateItem, Template},
    Amount, Tag,
};

use crate::lib::date::{Date, Month};

#[derive(Debug)]
pub enum AstItem {
    Entry(Date, Entry),
    Instance(Date, Instance),
    Template(String, Template),
}

use pest::{
    error::{Error, ErrorVariant},
    iterators::{Pair, Pairs},
};

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

macro_rules! decapitate {
    ( $node:expr ) => {{
        let mut items = $node.into_inner().into_iter();
        let fst = items.next().unwrap_or_else(|| panic!("No head"));
        (fst, items)
    }};
}

macro_rules! pair {
    ( $node:expr ) => {{
        let mut items = $node.into_inner().into_iter();
        let fst = items.next().unwrap_or_else(|| panic!("No 1st"));
        let snd = items.next().unwrap_or_else(|| panic!("No 2nd"));
        assert!(items.next().is_none());
        (fst, snd)
    }};
}

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

macro_rules! parse_usize {
    ( $node:expr ) => {
        $node.as_str().parse::<usize>().unwrap()
    };
}

macro_rules! parse_amount {
    ( $node:expr ) => {
        ($node.as_str().parse::<f64>().unwrap() * 100.0).round() as isize
    };
}

macro_rules! as_string {
    ( $node:expr ) => {
        $node.as_str().to_string()
    };
}

macro_rules! set_or_fail {
    ( $var:expr, $val:expr, $name:expr, $loc:expr ) => {{
        if $var.is_some() {
            failure!(format!("Attempt to override {}", $name), $loc)
        }
        $var = Some($val);
    }};
}
    let mut ast = Vec::new();
    for pair in pairs {
        match pair.as_rule() {
            Rule::item => {
                for item in pair.into_inner() {
                    match item.as_rule() {
                        Rule::comment => (),
                        Rule::template_descriptor => match validate_template(item) {
                            None => return None,
                            Some((name, templ)) => {
                                println!("{:#?}", templ);
                                ast.push(AstItem::Template(name, templ));
                            }
                        },
                        Rule::entries_year => {
                            let mut items = item.into_inner().into_iter();
                            let item = items.next().unwrap();
                            assert_eq!(item.as_rule(), Rule::marker_year);
                            let year = item.as_str().parse::<usize>()
                                .unwrap();
                            match validate_year(year, items.collect::<Vec<_>>()) {
                                None => return None,
                                Some(items) => {
                                    for item in items {
                                        ast.push(item);
                                    }
                                }
                            }
                        }
                        _ => unreachable!(),
                    }
                }
            }
            _ => unreachable!(),
        }
    }
    Some(ast)
}

fn validate_template(pair: Pair<'_, Rule>) -> Option<(String, Template)> {
    let loc = pair.as_span().clone();
    let mut pairs = pair.into_inner().into_iter();
    let identifier: String = {
        let id = pairs.next().unwrap();
        assert_eq!(id.as_rule(), Rule::identifier);
        id.as_str().to_string()
    };
    let item = pairs.next().unwrap();
    let (arguments, item) = {
        if item.as_rule() == Rule::template_args {
            (
                match validate_args(item.into_inner()) {
                    None => return None,
                    Some(args) => args,
                },
                pairs.next().unwrap(),
            )
        } else {
            (Vec::new(), item)
        }
    };
    assert_eq!(item.as_rule(), Rule::template_expansion_contents);
    let mut value: Option<AmountTemplate> = None;
    let mut cat: Option<Category> = None;
    let mut span: Option<Span> = None;
    let mut tag: Option<TagTemplate> = None;
    for sub in item.into_inner() {
        match sub.as_rule() {
            Rule::template_val => {
                if value.is_some() {
                    let msg = "Attempt to override val";
                    failure(msg, sub.as_span());
                    return None;
                }
                let amount = sub.into_inner().into_iter().next().unwrap();
                assert_eq!(amount.as_rule(), Rule::template_money_amount);
                match validate_template_amount(amount.into_inner().into_iter().next().unwrap()) {
                    None => return None,
                    Some(amount) => value = Some(amount),
                }
            }
            Rule::entry_type => {
                if cat.is_some() {
                    let msg = "Attempt to override type";
                    failure(msg, sub.as_span());
                    return None;
                }
                match validate_cat(sub.into_inner().into_iter().next().unwrap()) {
                    None => return None,
                    Some(c) => cat = Some(c),
                }
            }
            Rule::entry_span => {
                if span.is_some() {
                    let msg = "Attempt to override span";
                    failure(msg, sub.as_span());
                    return None;
                }
                match validate_span(sub) {
                    None => return None,
                    Some(s) => span = Some(s),
                }
            }
            Rule::template_tag => {
                let loc = sub.as_span().clone();
                if tag.is_some() {
                    let msg = "Attempt to override tag";
                    failure(msg, loc);
                    return None;
                }
                match validate_template_tag(sub.into_inner().into_iter().next().unwrap()) {
                    None => return None,
                    Some(t) => tag = Some(t),
                }
            }
            _ => unreachable!(),
        }
    }
    let value = match value {
        Some(v) => v,
        None => {
            failure("val is unspecified", loc);
            return None;
        }
    };
    let cat = match cat {
        Some(c) => c,
        None => {
            failure("cat is unspecified", loc);
            return None;
        }
    };
    let span = match span {
        Some(s) => s,
        None => {
            failure("span is unspecified", loc);
            return None;
        }
    };
    let tag = match tag {
        Some(t) => t,
        None => {
            failure("tag is unspecified", loc);
            return None;
        }
    };
    Some((
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

fn validate_args(pairs: Pairs<'_, Rule>) -> Option<Vec<(String, Option<Arg>)>> {
    let mut args = Vec::new();
    for pair in pairs {
        match validate_arg(pair) {
            None => return None,
            Some(arg) => args.push(arg),
        }
    }
    Some(args)
}

fn validate_arg(pair: Pair<'_, Rule>) -> Option<(String, Option<Arg>)> {
    match pair.as_rule() {
        Rule::template_positional_arg => {
            let name = pair.as_str().to_string();
            Some((name, None))
        }
        Rule::template_named_arg => {
            let mut items = pair.into_inner().into_iter();
            let name = items.next().unwrap().as_str().to_string();
            let default = {
                let item = items.next().unwrap();
                match item.as_rule() {
                    Rule::money_amount => match validate_amount(item) {
                        Some(amount) => Arg::Amount(amount),
                        None => return None,
                    },
                    Rule::tag_text => Arg::Tag(Tag(item
                        .into_inner()
                        .into_iter()
                        .next()
                        .unwrap()
                        .as_str()
                        .to_string())),
                    _ => {
                        unreachable!()
                    },
                }
            };
            Some((name, Some(default)))
        }
        _ => unreachable!(),
    }
}

fn validate_amount(item: Pair<'_, Rule>) -> Option<Amount> {
    assert_eq!(item.as_rule(), Rule::money_amount);
    item.as_str()
        .parse::<f64>()
        .ok()
        .map(|f| Amount((f * 100.0) as isize))
}

fn validate_template_amount(pair: Pair<'_, Rule>) -> Option<AmountTemplate> {
    let (sign, pair) = match pair.as_rule() {
        Rule::builtin_neg => (false, pair.into_inner().into_iter().next().unwrap()),
        _ => (true, pair),
    };
    let items = match pair.as_rule() {
        Rule::builtin_sum => pair
            .into_inner()
            .into_iter()
            .next()
            .unwrap()
            .into_inner()
            .into_iter()
            .map(|it| it.into_inner().into_iter().next().unwrap())
            .collect::<Vec<_>>(),
        _ => vec![pair],
    };
    let mut sum = Vec::new();
    for item in items {
        match item.as_rule() {
            Rule::money_amount => {
                sum.push(AmountTemplateItem::Cst(validate_amount(item).unwrap()));
            }
            Rule::template_arg_expand => sum.push(AmountTemplateItem::Arg(
                item.into_inner()
                    .into_iter()
                    .next()
                    .unwrap()
                    .as_str()
                    .to_string(),
            )),
            _ => unreachable!(),
        }
    }
    Some(AmountTemplate { sign, sum })
}

fn validate_cat(pair: Pair<'_, Rule>) -> Option<Category> {
    use Category::*;
    Some(match pair.as_str() {
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

fn validate_span(pair: Pair<'_, Rule>) -> Option<Span> {
    let mut pair = pair
        .into_inner()
        .into_iter()
        .next()
        .unwrap()
        .into_inner()
        .into_iter()
        .peekable();
    use Duration::*;
    println!("{:?}", pair);
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
    let count = pair
        .next()
        .map(|it| it.as_str().parse::<usize>().unwrap())
        .unwrap_or(1);
    Some(Span {
        duration,
        window: window.unwrap_or(Current),
        count,
    })
}

fn validate_template_tag(pair: Pair<'_, Rule>) -> Option<TagTemplate> {
    let concat = match pair.as_rule() {
        Rule::builtin_concat => pair
            .into_inner()
            .into_iter()
            .next()
            .unwrap()
            .into_inner()
            .into_iter()
            .map(|it| {
                assert_eq!(it.as_rule(), Rule::template_string);
                it.into_inner().into_iter().next().unwrap()
            })
            .collect::<Vec<_>>(),
        Rule::tag_text => vec![pair],
        _ => pair.into_inner().into_iter().collect::<Vec<_>>(),
    };
    let mut strs = Vec::new();
    use TagTemplateItem::*;
    println!("{:#?}", concat);
    for item in concat {
        strs.push(match item.as_rule() {
            Rule::tag_text => Raw(Tag(item
                .into_inner()
                .into_iter()
                .next()
                .unwrap()
                .as_str()
                .to_string())),
            Rule::template_arg_expand => Arg(item
                .into_inner()
                .into_iter()
                .next()
                .unwrap()
                .as_str()
                .to_string()),
            Rule::template_time => match item.as_str() {
                "@Day" => Day,
                "@Month" => Month,
                "@Year" => Year,
                _ => unreachable!(),
            },
            _ => unreachable!(),
        });
    }
    Some(TagTemplate(strs))
}

fn validate_year(year: usize, pairs: Vec<Pair<'_, Rule>>) -> Option<Vec<AstItem>> {
    let mut v = Vec::new();
    for pair in pairs {
        assert_eq!(pair.as_rule(), Rule::entries_month);
        let mut items = pair.into_inner().into_iter();
        let fst = items.next().unwrap();
        let month = Month::from(fst.as_str());
        let items = validate_month(year, month, items.collect::<Vec<_>>())?;
        for item in items {
            v.push(item);
        } 
    }
    Some(v)
}

fn validate_month(year: usize, month: Month, pairs: Vec<Pair<'_, Rule>>) -> Option<Vec<AstItem>> {
    let mut v = Vec::new();
    for pair in pairs {
        assert_eq!(pair.as_rule(), Rule::entries_day);
        let mut items = pair.into_inner().into_iter();
        let fst = items.next().unwrap();
        let day = fst.as_str().parse::<usize>().unwrap();
        match Date::from(year, month, day) {
            Ok(date) => {
                let items = validate_day(date, items.collect::<Vec<_>>())?;
                for item in items {
                    v.push(item);
                }
            }
            Err(err) => {
                println!("{}", err);
                return None;
            }
        }
    }
    Some(v)
}

fn validate_day(date: Date, pairs: Vec<Pair<'_, Rule>>) -> Option<Vec<AstItem>> {
    println!("{:?}", date);
    let mut v = Vec::new();
    for pair in pairs {
        assert_eq!(pair.as_rule(), Rule::entry);
        let entry = pair.into_inner().into_iter().next().unwrap();
        match entry.as_rule() {
            Rule::expand_entry => {
                let res = validate_expand_entry(entry.into_inner())?;
                v.push(AstItem::Instance(date.clone(), res));
            }
            Rule::plain_entry => {
                let res = validate_plain_entry(entry.into_inner())?;
                v.push(AstItem::Entry(date.clone(), res));
            }
            _ => unreachable!(),
        }
    }
    Some(v)
}

fn validate_expand_entry(pairs: Pairs<'_, Rule>) -> Option<Instance> {
    unimplemented!()
}

fn validate_plain_entry(pairs: Pairs<'_, Rule>) -> Option<Entry> {
    unimplemented!()
}
