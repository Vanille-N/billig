//! Pretty-printing facility for error messages
//!
//! In fairness, this is mostly a wrapper around `pest::error::Error::new_from_span`,
//! the difficult part of the formatting is handled and `Error` only adds aggregation
//! of messages as well as colored output.
//!
//! # Example
//!
//! ```rust
//! errs.make("Unused argument")
//!     .nonfatal()
//!     .span(inst_loc, format!("in instanciation of '{}'", inst_name))
//!     .text(format!("Argument '{}' is provided but not used", arg_name))
//!     .span(templ_loc, "defined here")
//!     .hint("remove argument or use in template")
//! ```
//!
//! ```txt
//! --> Warning: Unused argument
//!  |     --> ../examples/failures/unused.bil:10:13
//!  |      |
//!  |   10 |         01: !self_sufficient 0 other="";
//!  |      |             ^-------------------------^
//!  |      |
//!  |      = in instanciation of 'self_sufficient'
//!  |  Argument 'other' is provided but not used
//!  |     --> ../examples/failures/unused.bil:1:1
//!  |      |
//!  |    1 | !self_sufficient unused extra="" {
//!  |      | ...
//!  |    6 | }
//!  |      | ^
//!  |      |
//!  |      = defined here
//!  |      ? hint: remove argument or use in template
//! ```

/// Location of an error
///
/// Contains information on the file in which the error
/// occured and the precise span within that file
pub type Loc<'i> = (&'i str, pest::Span<'i>);

use crate::load::parse::Rule;

/// Report for a single error
///
/// All messages (`label` passed with `new`, arguments of `with_hint`
/// and `with_text`) should fit in a single line.
///
/// ```rust
/// // NO
/// errs.new("Fatal failure\ngeneral message\nspanning several lines\nhint to fix\nnote\nsee
/// documentation")
///
/// // YES
/// errs.make("Fatal failure")
///     .text("general message")
///     .text("spanning several lines")
///     .hint("hint to fix")
///     .hint("note")
///     .text("see documentation")
/// ```
#[must_use]
#[derive(Debug)]
pub struct Error {
    /// determines the error label (warning/error) and the color (yellow/red)
    fatal: bool,
    /// name of the error
    label: String,
    /// at which point of the contents is the counter
    items: Vec<Item>,
}

/// Kinds of items that can be added to an error report
#[derive(Debug)]
enum Item {
    /// code block
    Block(pest::error::Error<Rule>),
    /// important message
    Text(String),
    /// recommendations for fixes
    Hint(String),
}

/// A collection of errors
///
/// Typically to keep record of all errors detected in one file,
/// but the structure itself makes no assumption regarding the
/// spatial or semantic relationship between these errors
#[must_use]
#[derive(Debug)]
pub struct Record {
    /// how many are errors in the rest are warnings
    /// counts only `contents[..contents.len()-2]`
    fatal: usize,
    contents: Vec<Error>,
}

impl Error {
    /// Add a pre-existing error (e.g. to build from a parsing error)
    pub fn from(&mut self, err: pest::error::Error<Rule>) -> &mut Self {
        self.items
            .push(Item::Block(err.renamed_rules(rule_rename)));
        self
    }
}

impl Error {
    /// Create a new error
    pub fn new<S>(msg: S) -> Self
    where
        S: ToString,
    {
        Self {
            fatal: true,
            label: msg.to_string(),
            items: Vec::new(),
        }
    }

    /// Mark as a warning rather that a fatal error
    pub fn nonfatal(&mut self) -> &mut Self {
        self.fatal = false;
        self
    }

    /// Add a code block and its associated message
    pub fn span<S>(&mut self, loc: &Loc, msg: S) -> &mut Self
    where
        S: ToString,
    {
        self.items.push(Item::Block(
            pest::error::Error::new_from_span(
                pest::error::ErrorVariant::CustomError {
                    message: msg.to_string(),
                },
                loc.1.clone(),
            )
            .with_path(&loc.0.to_string()),
        ));
        self
    }

    /// Add an important note
    pub fn text<S>(&mut self, msg: S) -> &mut Self
    where
        S: ToString,
    {
        self.items.push(Item::Text(msg.to_string()));
        self
    }

    /// Add a hint on how to fix
    pub fn hint<S>(&mut self, msg: S) -> &mut Self
    where
        S: ToString,
    {
        self.items.push(Item::Hint(msg.to_string()));
        self
    }
}

impl Record {
    /// Initialize a new pool of errors (e.g. to record errors from another file)
    pub fn new() -> Self {
        Self {
            fatal: 0,
            contents: Vec::new(),
        }
    }

    /// Checks if any of the recorded errors are fatal
    pub fn is_fatal(&self) -> bool {
        self.fatal > 0 || self.last_is_fatal()
    }

    fn last_is_fatal(&self) -> bool {
        self.contents.last().map(|e| e.fatal).unwrap_or(false)
    }

    /// Number of fatal errors
    pub fn count_errors(&self) -> usize {
        self.fatal + if self.last_is_fatal() { 1 } else { 0 }
    }

    /// Number of nonfatal errors
    pub fn count_warnings(&self) -> usize {
        self.contents.len() - self.count_errors()
    }

    /// Add a new error to the pool
    pub fn make<S>(&mut self, msg: S) -> &mut Error
    where
        S: ToString,
    {
        if self.last_is_fatal() {
            self.fatal += 1;
        }
        self.contents.push(Error::new(msg));
        self.contents.last_mut().unwrap()
    }
}

const RED: &str = "\x1b[0;91;1m";
const YELLOW: &str = "\x1b[0;93;1m";
const BLUE: &str = "\x1b[0;96;1m";
const WHITE: &str = "\x1b[0;1m";
const NONE: &str = "\x1b[0m";

use std::fmt;
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (color, header) = if self.fatal {
            (RED, "--> Error")
        } else {
            (YELLOW, "--> Warning")
        };
        writeln!(f, "{}{}:{} {}{}", color, header, WHITE, self.label, NONE)?;
        for item in &self.items {
            match item {
                Item::Block(err) => {
                    let mut align = "   ".to_string();
                    let mut align_found = false;
                    for line in format!("{}", err).split('\n') {
                        write!(
                            f,
                            " {}|{}  {}",
                            color,
                            if align_found { &align } else { "" },
                            BLUE
                        )?;
                        for c in line.chars() {
                            match c {
                                '-' if !align_found => {
                                    align_found = true;
                                    write!(f, "{}-", align)?;
                                }
                                ' ' if !align_found => {
                                    align.pop();
                                    write!(f, " ")?;
                                }
                                '|' => write!(f, "|{}", NONE)?,
                                '=' => write!(f, "={}", NONE)?,
                                '^' => write!(f, "{}^", color)?,
                                '␊' => (), // pest::errors::Error does some weird display of line endings
                                _ => write!(f, "{}", c)?,
                            }
                        }
                        writeln!(f)?;
                    }
                }
                Item::Text(txt) => {
                    writeln!(f, " {}|  {}{}{}", color, WHITE, txt, NONE)?;
                }
                Item::Hint(txt) => {
                    writeln!(f, " {}|      {}? hint: {}{}", color, BLUE, NONE, txt)?;
                }
            }
        }
        Ok(())
    }
}

impl fmt::Display for Record {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.contents.is_empty() {
            return Ok(());
        }
        let fatal = self.is_fatal();
        let count = if fatal {
            self.count_errors()
        } else {
            self.count_warnings()
        };
        let color = if fatal { RED } else { YELLOW };
        let trunc = 10;
        for err in self
            .contents
            .iter()
            .filter(|err| err.fatal == fatal)
            .take(trunc)
        {
            // only print errors with the maximum fatality
            writeln!(f, "{}", err)?;
        }
        if count > trunc {
            writeln!(f, "{} And {} more.", color, count - trunc)?;
        }
        let plural = if count > 1 { "s" } else { "" };
        if fatal {
            writeln!(
                f,
                "{}Fatal: {}{} error{} emitted{}",
                color, WHITE, count, plural, NONE
            )?;
        } else {
            writeln!(
                f,
                "{}Nonfatal: {}{} warning{} emitted{}",
                color, WHITE, count, plural, NONE
            )?;
        }
        Ok(())
    }
}

fn rule_rename(r: &Rule) -> String {
        String::from(match r {
            Rule::EOI => "EOF",
            Rule::COMMENT => "a comment",
            Rule::digit => "a digit (0..9)",
            Rule::number => "a number",
            Rule::nonzero => "a non-null number",
            Rule::comma => "a comma (',') separator",
            Rule::whitespace => "at least one whitespace",
            Rule::semicolon => "a semicolon (';') separator",
            Rule::colon => "a colon (':') separator",
            Rule::marker_year => "a year marker ('YYYY:')",
            Rule::marker_month => "a month marker ('Jan:' ... 'Dec:')",
            Rule::marker_day => "a 1- or 2-digit day number",
            Rule::money_amount => "a monetary value ('XXX.XX')",
            Rule::tag_text => "a tag ('\"foo\"')",
            Rule::string => "a string of non-'\"' characters",
            Rule::identifier => "an identifier composed of a..zA..Z-_",
            Rule::span_value => "a number",
            Rule::entry_val => "a 'val' field descriptor",
            Rule::entry_type => "a 'type' field descriptor",
            Rule::entry_span => "a 'span' field descriptor",
            Rule::entry_tag => "a 'tag' field descriptor",
            Rule::entry_item => "any field descriptor",
            Rule::positional_arg => "a value for a positional argument",
            Rule::named_arg => "a name=value named argument pair",
            Rule::arguments => "a sequence of whitespace-separated argument instances",
            Rule::expand_entry => "a template expansion",
            Rule::plain_entry => "an entry composed of field descriptors",
            Rule::entry => "an explicit entry or a template expansion",
            Rule::entries_day => "a sequence of entries for the same day",
            Rule::entries_month => "a sequence of entries for the same month",
            Rule::entries_year => "a sequence of entries for the same year",
            Rule::template_time => "one of @Day, @Month, @Year, @Date, @Weekday",
            Rule::template_arg_expand => "an argument expansion *foo",
            Rule::template_value => "a monetary value or an argument expansion",
            Rule::template_string => "a tag or an argument expansion or a builtin date indicator",
            Rule::template_value_args => "arguments for a summation builtin",
            Rule::template_string_args => "a list of strings or items to concatenate",
            Rule::builtin_neg => "the negation of a value",
            Rule::template_money_amount => "a template for the 'val' field",
            Rule::template_val => "a 'val' field template descriptor",
            Rule::template_tag => "a 'tag' field template descriptor",
            Rule::template_entry => "a field descriptor",
            Rule::template_expansion_contents => "a sequence of field descriptors",
            Rule::template_positional_arg => "a positional template argument",
            Rule::template_named_arg => "a named template argument with a default value",
            Rule::template_args => "a sequence of template arguments",
            Rule::template_descriptor => "a template description",
            Rule::item => "a template description or a sequence of entries",
            Rule::program => "a sequence of template descriptions or sequences of entries",
            Rule::uppercase => "an uppercase letter (start of a month name)",
            Rule::lowercase => "a lowercase letter (rest of a month name)",
            Rule::month_date => "a date Mmm-DD or Mmm",
            Rule::full_date => "a date YYYY-Mmm-DD or YYYY-Mmm or YYYY",
            Rule::partial_date => "a date YYYY-Mmm-DD or YYYY-Mmm or YYYY or Mmm-DD or Mmm or DD",
            Rule::period_after => "a period [start..]",
            Rule::period_before => "a period [..end?]",
            Rule::period_between => "a period [start..end]",
            Rule::period => "a period [start?..end?]",
            Rule::period_empty => "an empty period ()",
            Rule::period_only => "a period [start?..end?]",
            Rule::builtin => "a capitalized identifier",
            Rule::duration => "a duration Day, Week, Month or Year",
            Rule::window => "a window Curr, Post, Ante, Pred or Succ",
            Rule::expense_type => "an expense type Mov, Home, Tech, ...",
        })
    }

