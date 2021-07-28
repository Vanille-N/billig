//! Convert a reference to a file into a stream of AST items
//! (entries and templates)

#![allow(clippy::upper_case_acronyms)]

use pest::Parser;
use pest_derive::*;

/// Wrapper around Pest's `Pair`
type Pair<'i> = pest::iterators::Pair<'i, Rule>;
/// Wrapper around Pest's `Pairs`
type Pairs<'i> = pest::iterators::Pairs<'i, Rule>;

use crate::lib::{
    date::{Date, Month},
    entry::{self, Amount, Category, Entry, Span, Tag},
};
use crate::load::{
    error,
    template::models::{self, Arg, Instance, Template},
};

/// Convenient exports
pub mod ast {
    pub use super::{Ast, AstItem as Item};
}

/// Pest-generated parser
#[derive(Parser)]
#[grammar = "load/billig.pest"]
pub struct BilligParser;

/// A collection of AST items, i.e. entries and template definitions
pub type Ast<'i> = Vec<AstItem<'i>>;

/// Each item of the file
#[derive(Debug)]
pub enum AstItem<'i> {
    /// an explicit entry with its date
    Entry(Entry),
    /// a template expansion with its date
    Instance(Date, Instance<'i>),
    /// a template definition
    Template(&'i str, Template<'i>),
    /// an external file import
    Import(&'i str),
}

struct Once<'i, T> {
    name: &'i str,
    hint: &'i str,
    loc: &'i error::Loc<'i>,
    valid: bool,
    data: Option<T>,
}

impl<'i, T> Once<'i, T> {
    fn new(name: &'i str, hint: &'i str, loc: &'i error::Loc) -> Self {
        Self {
            name,
            hint,
            loc,
            valid: true,
            data: None,
        }
    }

    fn try_set(&mut self, val: T, errs: &mut error::Record) {
        if self.data.is_some() {
            errs.make("Duplicate field definition")
                .span(self.loc, format!("attempt to override {}", self.name))
                .text("Each field may only be defined once")
                .hint("remove one of the field definitions");
            self.valid = false;
        }
        self.data = Some(val);
    }

    fn try_get(self, errs: &mut error::Record) -> Option<T> {
        if self.valid {
            if self.data.is_none() {
                errs.make("Missing field definition")
                    .span(self.loc, format!("'{}' may not be omitted", self.name))
                    .text("Each field must be defined once")
                    .hint(format!(
                        "add definition for the missing field: '{} {}'",
                        self.name, self.hint
                    ));
                None
            } else {
                self.data
            }
        } else {
            None
        }
    }
}

/// Get the contents of file `path`
///
/// The return value may be non-empty even if some errors (including fatal ones) occured.
/// More specifically, return value is likely (but not guaranteed in the long term) to
/// contain all items that parsed correctly.
///
/// Caller should determine the success of this function not through its return value
/// but by querying `errs` (e.g. by checking `errs.is_fatal()` or `errs.count_errors()`)
pub fn extract<'i>(path: &'i str, errs: &mut error::Record, contents: &'i str) -> Ast<'i> {
    match BilligParser::parse(Rule::program, contents) {
        Ok(contents) => validate(path, errs, contents),
        Err(e) => {errs.make("Parsing failure").from(e.with_path(path));
            Vec::new()
    }
    }
}

// extract contents of wrapper rule
macro_rules! subrule {
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
        // safe to .unwrap() because the grammar validated it already
        Amount(($node.as_str().parse::<f64>().unwrap() * 100.0).round() as isize)
    };
}

/// Check all items
///
/// Sequentially validates each entry or template, records errors, accumulates the
/// correct ones into the return value.
pub fn validate<'i>(path: &'i str, errs: &mut error::Record, pairs: Pairs<'i>) -> Ast<'i> {
    let mut ast = Vec::new();
    'pairs: for pair in pairs {
        match pair.as_rule() {
            Rule::template_descriptor => {
                let (name, templ) = match validate_template(path, errs, pair) {
                    Some(x) => x,
                    None => continue 'pairs,
                };
                ast.push(AstItem::Template(name, templ));
            }
            Rule::entries_year => {
                let (head, body) = decapitate!(pair);
                assert_eq!(head.as_rule(), Rule::marker_year);
                let year = parse_usize!(head);
                let items = validate_year(path, errs, year, body.collect::<Vec<_>>());
                for item in items {
                    ast.push(item);
                }
            }
            Rule::import => {
                ast.push(AstItem::Import(pair.into_inner().as_str()));
                //let relative = pair.into_inner().as_str();
                //let mut file = std::path::PathBuf::from(path);
                //file.pop();
                //file.push(relative);
                //let filename = file.to_str().unwrap();
                //let contents = std::fs::read_to_string(&file).expect(&format!("File '{}' not found", filename));
                //println!("Reading data from '{}'", filename);
                //extract(filename, errs, &contents, ast);
            }
            Rule::EOI => break,
            _ => unreachable!(),
        }
    }
    ast
}

/// Check that a template is valid
///
/// This can raise errors since the grammar can't ensure that no
/// duplicate field is present or that no field definition is missing
fn validate_template<'i>(
    path: &'i str,
    errs: &mut error::Record,
    pair: Pair<'i>,
) -> Option<(&'i str, Template<'i>)> {
    let loc = (path, pair.as_span().clone());
    let (id, args, body) = triplet!(pair);
    assert_eq!(id.as_rule(), Rule::identifier);
    let identifier = id.as_str();
    assert_eq!(args.as_rule(), Rule::template_args);
    let (positional, named) = read_args(args.into_inner());
    assert_eq!(body.as_rule(), Rule::template_expansion_contents);
    let mut value = Once::new("val", "42.69", &loc);
    let mut cat = Once::new("type", "Food", &loc);
    let mut span = Once::new("span", "Week<Post> 2", &loc);
    let mut tag = Once::new("tag", "Some information", &loc);
    for sub in body.into_inner() {
        match sub.as_rule() {
            Rule::builtin => {
                let loc = (path, sub.as_span().clone());
                if let Ok(c) = sub.as_str().parse::<entry::Category>() {
                    cat.try_set(c, errs);
                } else if let Ok(d) = sub.as_str().parse::<entry::Duration>() {
                    span.try_set(Span::from(d, entry::Window::Posterior, 1), errs);
                } else {
                    errs.make("Invalid builtin of ambiguous nature")
                        .span(&loc, "provided here")
                        .text("This keyword is not recognized")
                        .hint("maybe you meant one of Food, Com, Mov, Home, ...")
                        .hint("or maybe try Day, Week, Month, Year");
                    return None;
                }
            }
            Rule::template_money_amount => {
                value.try_set(read_template_amount(subrule!(sub)), errs);
            }
            Rule::expense_type => {
                cat.try_set(validate_cat(path, errs, sub)?, errs);
            }
            Rule::span_value => {
                span.try_set(validate_span(path, errs, sub)?, errs);
            }
            Rule::template_tag => {
                tag.try_set(read_template_tag(subrule!(sub)), errs);
            }
            Rule::money_amount => {
                value.try_set(read_template_amount(sub), errs);
            }
            Rule::string => {
                tag.try_set(read_template_tag(sub), errs);
            }
            _ => unreachable!(),
        }
    }
    let value = value.try_get(errs)?;
    let cat = cat.try_get(errs)?;
    let span = span.try_get(errs)?;
    let tag = tag.try_get(errs)?;
    Some((
        identifier,
        Template::new(positional, named, value, cat, span, tag, loc),
    ))
}

/// Parse list of arguments
///
/// Grammar ensures this cannot fail
fn read_args(pairs: Pairs) -> (Vec<&str>, Vec<(&str, Arg)>) {
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

/// Parse a single positional or named argument
///
/// Grammar ensures this cannot fail
fn read_arg(pair: Pair) -> (&str, Option<Arg>) {
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
                    Rule::string => Arg::Tag(default.as_str()),
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

/// Parse an amount of money
///
/// Grammar ensures this cannot fail, as accepted amounts
/// are a subset of valid float representations
fn read_amount(item: Pair) -> Amount {
    assert_eq!(item.as_rule(), Rule::money_amount);
    parse_amount!(item)
}

/// Parse a template item that expands to an amount
///
/// May contain `@Neg`, then possibly `@Sum`, then a list of either values
/// or argument identifiers. Grammar ensures this cannot fail.
fn read_template_amount(pair: Pair) -> models::amount::Template {
    let (sign, pair) = match pair.as_rule() {
        Rule::builtin_neg => (false, subrule!(pair)),
        _ => (true, pair),
    };
    let items = match pair.as_rule() {
        Rule::money_amount => vec![pair],
        _ => pair.into_inner().into_iter().collect::<Vec<_>>(),
    };
    use models::amount::*;
    let mut sum = Template::new(sign);
    for item in items {
        match item.as_rule() {
            Rule::money_amount => {
                sum.push(Item::Cst(read_amount(item)));
            }
            Rule::identifier => sum.push(Item::Arg(item.as_str())),
            _ => unreachable!(),
        }
    }
    sum
}

/// Parse an expense category
fn validate_cat(path: &str, errs: &mut error::Record, pair: Pair) -> Option<Category> {
    let loc = (path, pair.as_span().clone());
    match pair.as_str().parse::<entry::Category>() {
        Ok(category) => Some(category),
        Err(()) => {
            errs.make("Invalid category")
                .span(&loc, "provided here")
                .text(format!("'{}' is not a valid expense type", pair.as_str()))
                .hint("use one of Home, Food, Move, Tech, Pay, Pro, Fun");
            None
        }
    }
}

/// Parse a span (length, window, count)
fn validate_span(path: &str, errs: &mut error::Record, pair: Pair) -> Option<Span> {
    let mut pair = pair.into_inner().into_iter().peekable();
    let item_dur = pair.next().unwrap();
    let loc = (path, item_dur.as_span().clone());
    let duration = match item_dur.as_str().parse::<entry::Duration>() {
        Ok(dur) => dur,
        Err(()) => {
            errs.make("Invalid duration")
                .span(&loc, "provided here")
                .text(format!("'{}' is not a valid duration", item_dur.as_str()))
                .hint("use one of Day, Week, Month, Year");
            return None;
        }
    };
    let has_window = pair.peek().map(|it| it.as_rule() == Rule::window).unwrap_or(false);
    let window = if has_window {
        let win_rule = pair.next().unwrap();
        let loc = (path, win_rule.as_span().clone());
        match win_rule.as_str().parse::<entry::Window>() {
            Ok(win) => win,
            Err(()) => {
                errs.make("Invalid window")
                    .span(&loc, "provided here")
                    .text(format!("'{}' is not a valid window", win_rule.as_str()))
                    .hint("use one of Curr, Ante, Pred, Post, Succ");
                return None;
            }
        }
    } else {
        entry::Window::Posterior
    };
    let count = pair.next().map(|it| parse_usize!(it)).unwrap_or(1);
    Some(Span::from(duration, window, count))
}

/// Parse a template item that expands to a tag
///
/// Grammar ensures this cannot fail, as raw tags are valid strings,
/// arguments are valid identifiers, and builtin placeholders (`@Day`, `@Date`, ...)
/// have keyword status
fn read_template_tag(pair: Pair) -> models::tag::Template {
    let concat = pair.into_inner().into_iter().collect::<Vec<_>>();
    use models::tag::*;
    let mut strs = Template::new();
    for item in concat {
        strs.push(match item.as_rule() {
            Rule::string => Item::Raw(item.as_str()),
            Rule::identifier => Item::Arg(item.as_str()),
            Rule::template_time => match item.as_str() {
                "@Day" => Item::Day,
                "@Month" => Item::Month,
                "@Year" => Item::Year,
                "@Date" => Item::Date,
                "@Weekday" => Item::Weekday,
                _ => unreachable!(),
            },
            _ => unreachable!(),
        });
    }
    strs
}

/// Parse a series of entries registered for the same year
///
/// The inner operation (`validate_month`) can produce errors
fn validate_year<'i>(
    path: &'i str,
    errs: &mut error::Record,
    year: usize,
    pairs: Vec<Pair<'i>>,
) -> Vec<AstItem<'i>> {
    let mut v = Vec::new();
    for pair in pairs {
        assert_eq!(pair.as_rule(), Rule::entries_month);
        let (month, rest) = decapitate!(pair);
        let loc = (path, month.as_span().clone());
        let month = match month.as_str().parse::<Month>() {
            Ok(month) => month,
            Err(()) => {
                errs.make("Invalid month")
                    .span(&loc, "provided here")
                    .hint("Months are 'Jan', 'Feb', ..., 'Dec'");
                continue;
            }
        };
        let items = validate_month(path, errs, year, month, rest.collect::<Vec<_>>());
        for item in items {
            v.push(item);
        }
    }
    v
}

/// Parse a series of entries registered for the same month
///
/// The inner operation (`validate_day`) and the date creation can both
/// produce errors
fn validate_month<'i>(
    path: &'i str,
    errs: &mut error::Record,
    year: usize,
    month: Month,
    pairs: Vec<Pair<'i>>,
) -> Vec<AstItem<'i>> {
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
                errs.make("Invalid date")
                    .span(&loc, "provided here")
                    .text(format!("{}", e))
                    .hint("choose a date that exists")
                    .hint(e.fix_hint());
                continue 'pairs; // error does not interrupt parsing
            }
        }
    }
    v
}

/// Parse a series of entries registered for the same day
///
/// One of the inner operations (`validate_plain_entry`) can produce errors
fn validate_day<'i>(
    path: &'i str,
    errs: &mut error::Record,
    date: Date,
    pairs: Vec<Pair<'i>>,
) -> Vec<AstItem<'i>> {
    let mut v = Vec::new();
    'pairs: for pair in pairs {
        let entry = subrule!(pair);
        let loc = (path, entry.as_span().clone());
        match entry.as_rule() {
            Rule::expand_entry => {
                let res = read_expand_entry(entry, loc);
                v.push(AstItem::Instance(date, res));
            }
            Rule::plain_entry => {
                let res = match validate_plain_entry(path, errs, date, entry) {
                    Some(x) => x,
                    None => continue 'pairs,
                };
                v.push(AstItem::Entry(res));
            }
            _ => unreachable!(),
        }
    }
    v
}

/// Parse a template instanciation
///
/// Grammar ensures this cannot fail (but it may produce errors
/// down the line during template expansion)
fn read_expand_entry<'i>(pairs: Pair<'i>, loc: error::Loc<'i>) -> Instance<'i> {
    let (label, args) = pair!(pairs);
    let label = label.as_str();
    let mut positional = Vec::new();
    let mut named = Vec::new();
    for arg in args.into_inner() {
        match arg.as_rule() {
            Rule::money_amount | Rule::string => {
                positional.push(read_value(arg));
            }
            Rule::named_arg => {
                let (name, value) = pair!(arg);
                let name = name.as_str();
                let value = read_value(value);
                named.push((name, value));
            }
            _ => unreachable!(),
        }
    }
    Instance::new(label, positional, named, loc)
}

/// Parse either an amount of money or a tag
///
/// Both of these types may appear as default values or as arguments
/// passed to a template instanciation
fn read_value(pair: Pair) -> Arg {
    match pair.as_rule() {
        Rule::money_amount => Arg::Amount(read_amount(pair)),
        Rule::string => Arg::Tag(pair.as_str()),
        _ => {
            unreachable!()
        }
    }
}

/// Parse an explicit entry (i.e. not a template instanciation)
///
/// This can fail since the grammar can't ensure that there is no duplicate field
/// definition or that there is no missing field
fn validate_plain_entry(path: &str, errs: &mut error::Record, date: Date, pair: Pair) -> Option<Entry> {
    let loc = (path, pair.as_span().clone());
    let mut value = Once::new("val", "42.69", &loc);
    let mut cat = Once::new("type", "Food", &loc);
    let mut span = Once::new("span", "Week<Post> 2", &loc);
    let mut tag = Once::new("tag", "Some information", &loc);
    for item in pair.into_inner() {
        match item.as_rule() {
            Rule::builtin => {
                if let Ok(c) = item.as_str().parse::<entry::Category>() {
                    cat.try_set(c, errs);
                } else if let Ok(d) = item.as_str().parse::<entry::Duration>() {
                    span.try_set(Span::from(d, entry::Window::Posterior, 1).period(date), errs);
                } else {
                    errs.make("Invalid builtin of ambiguous nature")
                        .span(&loc, "provided here")
                        .text("This keyword is not recognized")
                        .hint("maybe you meant one of Food, Com, Mov, Home, ...")
                        .hint("or maybe try Day, Week, Month, Year");
                    return None;
                }

            }
            Rule::money_amount => {
                value.try_set(parse_amount!(item), errs);
            }
            Rule::expense_type => {
                cat.try_set(validate_cat(path, errs, item)?, errs);
            }
            Rule::span_value => {
                span.try_set(validate_span(path, errs, item)?.period(date), errs);
            }
            Rule::string => {
                tag.try_set(Tag(item.as_str().to_string()), errs);
            }
            Rule::period => {
                use crate::lib::period::{self, PartialPeriod};
                let loc = (path, item.as_span().clone());
                let partial_period = period::validate_partial_period(path, errs, item.into_inner())?;
                let period = partial_period.make(errs, &loc, date)?.bounded(errs, &loc, date)?;
                span.try_set(period, errs);
            }
            _ => unreachable!("{:?}", item),
        }
    }
    let value = value.try_get(errs)?;
    let cat = cat.try_get(errs)?;
    let span = span.try_get(errs)?;
    let tag = tag.try_get(errs)?;
    Some(Entry::from(value, cat, span, tag))
}
