pub type Ast = Vec<AstItem>;

use crate::extract::{
    Amount,
    Tag,
    entry::{Entry, Category, Span},
    template::{Template, AmountTemplate, TagTemplate, AmountTemplateItem},
    instance::{Instance, Arg},
    parse::Rule,
};

#[derive(Debug)]
pub enum AstItem {
    Entry(Entry),
    Instance(Instance),
    Template(String, Template),
}

use pest::{
    iterators::{Pairs, Pair},
    error::{Error, ErrorVariant},
};

fn failure(msg: &str, span: pest::Span) {
    let err: Error<Rule> = Error::new_from_span(
        ErrorVariant::CustomError {
            message: String::from(msg),
        },
        span
    );
    println!("{}", err);
}


pub fn validate(pairs: Pairs<'_, Rule>) -> Option<Ast> {
    let mut ast = Vec::new();
    for pair in pairs {
        match pair.as_rule() {
            Rule::item => {
                for item in pair.into_inner() {
                    match item.as_rule() {
                        Rule::comment => (),
                        Rule::template_descriptor => {
                            match validate_template(item.into_inner()) {
                                None => return None,
                                Some((name, templ)) => {
                                    ast.push(AstItem::Template(name, templ));
                                }
                            }
                        }
                        _ => panic!("{:#?}", item.as_rule()),
                    }
                } 
            }
            _ => unreachable!(),
        }        
    }
    Some(ast)
}

fn validate_template(pairs: Pairs<'_, Rule>) -> Option<(String, Template)> {
    let mut pairs = pairs.into_iter();
    let identifier: String = {
        let id = pairs.next().unwrap();
        assert_eq!(id.as_rule(), Rule::identifier);
        id.as_str().to_string()
    };
    println!("{}", identifier);
    let item = pairs.next().unwrap();
    let (args, item) = {
        if item.as_rule() == Rule::template_args {
            (
                match validate_args(item.into_inner()) {
                    None => return None,
                    Some(args) => args,
                },
                pairs.next().unwrap()
            )
        } else {
            (
                Vec::new(),
                item
            )
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
                let span = sub.as_span().clone();
                let amount = sub.into_inner().into_iter().next().unwrap();
                assert_eq!(amount.as_rule(), Rule::template_money_amount);
                match validate_template_amount(amount.into_inner().into_iter().next().unwrap()) {
                    None => return None,
                    Some(amount) => {
                        if value.is_some() {
                            let msg = "Only one instance of each category is allowed";
                            failure(msg, span);
                            return None;
                        }
                        value = Some(amount);
                    }
                }
            }
            _ => panic!("{:?}", sub.as_rule()),
        }
    }
    None
}

fn validate_args(pairs: Pairs<'_, Rule>) -> Option<Vec<(String, Option<Arg>)>> {
    let mut args = Vec::new();
    for pair in pairs {
        assert_eq!(pair.as_rule(), Rule::template_arg);
        match validate_arg(pair.into_inner()) {
            None => return None,
            Some(arg) => args.push(arg),
        }
    }
    println!("{:?}", args);
    Some(args)
}

fn validate_arg(pairs: Pairs<'_, Rule>) -> Option<(String, Option<Arg>)> {
    let mut pairs = pairs.into_iter();
    let fst = pairs.next().unwrap();
    assert_eq!(fst.as_rule(), Rule::identifier);
    let name = fst.as_str().to_string();
    let default = match pairs.next() {
        Some(item) => {
            match item.as_rule() {
                Rule::money_amount => {
                    match validate_amount(item) {
                        Some(amount) => Some(Arg::Amount(amount)),
                        None => return None,
                    }
                }
                Rule::tag_text => {
                    Some(Arg::Tag(Tag(item.as_str().to_string())))
                }
                _ => unreachable!(),
            }
        }
        None => None,
    }; 
    Some((name, default))
}

fn validate_amount(item: Pair<'_, Rule>) -> Option<Amount> {
    assert_eq!(item.as_rule(), Rule::money_amount);
    item.as_str().parse::<f64>().ok().map(|f| Amount((f * 100.0) as isize))
}

fn validate_template_amount(pair: Pair<'_, Rule>) -> Option<AmountTemplate> {
    let (sign, pair) = match pair.as_rule() {
        Rule::builtin_neg => (false, pair.into_inner().into_iter().next().unwrap()),
        _ => (true, pair)
    };
    let items = match pair.as_rule() {
        Rule::builtin_sum => {
            pair.into_inner().into_iter()
                .next().unwrap()
                .into_inner().into_iter()
                .map(|it| it.into_inner().into_iter().next().unwrap())
                .collect::<Vec<_>>()
        }
        _ => vec![pair],
    };
    println!("{:#?}", items);
    let mut sum = Vec::new();
    for item in items {
        match item.as_rule() {
            Rule::money_amount => {
                sum.push(AmountTemplateItem::Cst(validate_amount(item).unwrap()));
            }
            Rule::template_arg_expand => {
                sum.push(AmountTemplateItem::Arg(item.into_inner().into_iter().next().unwrap().as_str().to_string()))
            }
            _ => unreachable!(),
        }
    }
    Some(AmountTemplate { sign, sum })
}
