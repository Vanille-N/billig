//! Pretty-printing facility for error messages
//!
//! In fairness, this is mostly a wrapper around `pest::error::Error::new_from_span`,
//! the difficult part of the formatting is handled and `Error` only adds aggregation
//! of messages as well as colored output.
//!
//! Note that using a `Record` is the only way to produce an error
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

use crate::lib::parse::Rule;

/// Location of an error
///
/// Contains information on the file in which the error
/// occured and the precise span within that file
pub type Loc<'i> = (&'i str, pest::Span<'i>);

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
#[derive(Debug, Default)]
pub struct Record {
    /// how many are errors in the rest are warnings
    /// counts only `contents[..contents.len()-2]`
    fatal: usize,
    contents: Vec<Error>,
}   

impl Error {
    /// Create a new error
    fn new<S>(msg: S) -> Self
    where S: ToString {
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

    /// Add a pre-existing error (e.g. to build from a parsing error)
    pub fn from(&mut self, err: pest::error::Error<Rule>) -> &mut Self {
        self.items.push(Item::Block(err.renamed_rules(rule_rename)));
        self
    }

    /// Add a code block and its associated message
    pub fn span<S>(&mut self, loc: &Loc, msg: S) -> &mut Self
    where S: ToString {
        self.items.push(Item::Block(pest::error::Error::new_from_span(
            pest::error::ErrorVariant::CustomError {
                message: msg.to_string(),
            },
            loc.1.clone(),
        ).with_path(&loc.0.to_string())));
        self
    }

    /// Add an important note
    pub fn text<S>(&mut self, msg: S) -> &mut Self
    where S: ToString {
        self.items.push(Item::Text(msg.to_string()));
        self
    }

    /// Add a hint on how to fix
    pub fn hint<S>(&mut self, msg: S) -> &mut Self
    where S: ToString {
        self.items.push(Item::Hint(msg.to_string()));
        self
    }
}

impl Record {
    /// Initialize a new pool of errors (e.g. to record errors from another file)
    pub fn new() -> Self {
        Self::default()
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
    where S: ToString {
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
                        write!(f, " {}|{}  {}", color, if align_found { &align } else { "" }, BLUE)?;
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
        let count = if fatal { self.count_errors() } else { self.count_warnings() };
        let color = if fatal { RED } else { YELLOW };
        let trunc = 10;
        for err in self.contents.iter().filter(|err| err.fatal == fatal).take(trunc) {
            // only print errors with the maximum fatality
            writeln!(f, "{}", err)?;
        }
        if count > trunc {
            writeln!(f, "{} And {} more.", color, count - trunc)?;
        }
        let plural = if count > 1 { "s" } else { "" };
        if fatal {
            writeln!(f, "{}Fatal: {}{} error{} emitted{}", color, WHITE, count, plural, NONE)?;
        } else {
            writeln!(f, "{}Nonfatal: {}{} warning{} emitted{}", color, WHITE, count, plural, NONE)?;
        }
        Ok(())
    }
}


/// Convert rule names to user-friendly information about their purpose
fn rule_rename(rule: &Rule) -> String {
    use Rule::*;
    String::from(match rule {
        EOI => "EOF",
        COMMENT => "a comment",
        digit => "a digit (0..9)",
        number => "a number",
        nonzero => "a non-null number",
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
        expense_type => "one of Pay, Food, Tech, Mov, Pro, Clean, Home",
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

