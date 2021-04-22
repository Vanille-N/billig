use crate::lib::parse::Rule;

pub type Loc<'i> = (&'i str, pest::Span<'i>);

#[must_use]
#[derive(Debug)]
pub struct Error {
    fatal: bool,
    label: String,
    items: Vec<ErrItem>,
}

#[derive(Debug)]
enum ErrItem {
    Block(pest::error::Error<Rule>),
    Text(String),
}

#[derive(Debug, Default)]
pub struct ErrorRecord {
    fatal: usize,
    contents: Vec<Error>,
}   

impl Error {
    pub fn new<S>(msg: S) -> Self
    where S: ToString {
        Self {
            fatal: true,
            label: msg.to_string(),
            items: Vec::new(),
        }
    }

    pub fn nonfatal(mut self) -> Self {
        self.fatal = false;
        self
    }

    pub fn with_error(mut self, err: pest::error::Error<Rule>) -> Self {
        self.items.push(ErrItem::Block(err.renamed_rules(rule_rename)));
        self
    }

    pub fn with_span<S>(mut self, loc: &Loc, msg: S) -> Self
    where S: ToString {
        self.items.push(ErrItem::Block(pest::error::Error::new_from_span(
            pest::error::ErrorVariant::CustomError {
                message: msg.to_string(),
            },
            loc.1.clone(),
        ).with_path(&loc.0.to_string())));
        self
    }

    pub fn with_message<S>(mut self, msg: S) -> Self
    where S: ToString {
        self.items.push(ErrItem::Text(msg.to_string()));
        self
    }

    pub fn register(self, record: &mut ErrorRecord) {
        record.register(self);
    }
}

impl ErrorRecord {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_fatal(&self) -> bool {
        self.fatal > 0
    }

    pub fn count_errors(&self) -> usize {
        self.fatal
    }

    pub fn count_warnings(&self) -> usize {
        self.contents.len() - self.fatal
    }

    pub fn count(&self) -> usize {
        self.contents.len()
    }

    pub fn register(&mut self, err: Error) {
        if err.fatal {
            self.fatal += 1;
        }
        self.contents.push(err);
    }
}


const RED: &'static str = "\x1b[0;91;1m";
const YLW: &'static str = "\x1b[0;93;1m";
const BLU: &'static str = "\x1b[0;36m";
const WHT: &'static str = "\x1b[0;1m";
const NONE: &'static str = "\x1b[0m";

use std::fmt;
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (color, header) = if self.fatal {
            (RED, "--> Error")
        } else {
            (YLW, "--> Warning")
        }; 
        write!(f, "{}{}:{} {}{}\n", color, header, WHT, self.label, NONE)?;
        for item in &self.items {
            match item {
                ErrItem::Block(err) => {
                    for line in format!("{}", err).split('\n') {
                        write!(f, "     {}|  {}", color, BLU)?;
                        for c in line.chars() {
                            match c {
                                '|' => write!(f, "|{}", NONE)?,
                                '=' => write!(f, "={}", NONE)?,
                                '^' => write!(f, "{}^", color)?,
                                _ => write!(f, "{}", c)?,
                            }
                        }
                        write!(f, "\n")?;
                    }
                }
                ErrItem::Text(txt) => {
                    write!(f, "     {}|  {}{}{}\n", color, WHT, txt, NONE)?;
                }
            }
        }
        Ok(())
    }
}

impl fmt::Display for ErrorRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fatal = self.is_fatal();
        let count = if fatal { self.count_errors() } else { self.count_warnings() };
        let color = if fatal { RED } else { YLW };
        let trunc = 10;
        for err in self.contents.iter().filter(|err| err.fatal == fatal).take(trunc) {
            // only print errors with the maximum fatality
            write!(f, "{}\n", err)?;
        }
        if count > trunc {
            write!(f, "{} And {} more.", color, count - trunc)?;
        }
        if fatal {
            write!(f, "{}Fatal: {}{} errors produced{}\n", color, WHT, count, NONE)?;
        } else {
            write!(f, "{}Nonfatal: {}{} warnings produced{}\n", color, WHT, count, NONE)?;
        }
        Ok(())
    }
}


fn rule_rename(rule: &Rule) -> String {
    use Rule::*;
    String::from(match rule {
        EOI => "EOF",
        COMMENT => "a comment",
        digit => "a digit (0..9)",
        number => "a number",
        comma => "a comma (',') separator",
        whitespace => "at least one whitespace",
        semicolon => "a semicolon (';') separator",
        colon => "a colon (':') separator",
        marker_year => "a year marker ('YYYY:')",
        marker_month => "a month marker ('Jan:' ... 'Dec:')",
        marker_day => "a day marker ('DD:')",
        money_amount => "a monetary value ('XXX.XX')",
        tag_text => "a tag ('\"foo\"')",
        string => "a string of non-'\"' characters",
        identifier => "an identifier composed of a..zA..Z-_",
        expense_type => "one of Pay, Food, Com, Mov, Pro, Clean, Home",
        span_duration => "one of Day, Week, Month, Year",
        span_window => "one of Ante, Pred, Curr, Succ, Post",
        span_value => "a number",
        entry_val => "a 'val' field descriptor",
        entry_type => "a 'type' field descriptor",
        entry_span => "a 'span' field descriptor",
        entry_tag => "a 'tag' field descriptor",
        entry_item => "any field descriptor",
        positional_arg => "a value for a positional argument",
        named_arg => "a name=value named argument pair",
        arguments => "a sequence of whitespace-separated argument instances",
        expand_entry => "a template expansion",
        plain_entry => "an entry composed of field descriptors", 
        entry => "an explicit entry or a template expansion",
        entries_day => "a sequence of entries for the same day",
        entries_month => "a sequence of entries for the same month",
        entries_year => "a sequence of entries for the same year",
        template_time => "one of @Day, @Month, @Year, @Date, @Weekday",
        template_arg_expand => "an argument expansion &foo",
        template_value => "a monetary value or an argument expansion",
        template_string => "a tag or an argument expansion or a builtin date indicator",
        template_value_args => "arguments for a summation builtin",
        template_string_args => "arguments for the @Concat builtin",
        builtin_neg => "the negation of a value",
        builtin_sum => "a summation of values",
        template_money_amount => "a template for the 'val' field",
        builtin_concat => "a concatenation of strings",
        template_val => "a 'val' field template descriptor",
        template_tag => "a 'tag' field template descriptor",
        template_entry => "a field descriptor",
        template_expansion_contents => "a sequence of field descriptors",
        template_positional_arg => "a positional template argument",
        template_named_arg => "a named template argument with a default value",
        template_args => "a sequence of template arguments",
        template_descriptor => "a template description",
        item => "a template description or a sequence of entries",
        program => "a sequence of template descriptions or sequences of entries",
    })
}

