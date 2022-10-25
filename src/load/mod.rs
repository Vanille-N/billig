pub mod error;
pub mod parse;
pub mod template;

use crate::util::{
    date::{Date, Interval},
    entry::Entry,
};

pub fn read_entries(filename: &str, errs: &mut error::Record) -> (Option<Vec<Entry>>, Interval<Date>) {
    let contents = match std::fs::read_to_string(filename) {
        Ok(contents) => contents,
        Err(_) => {
            errs.make("File not found")
                .text(format!("Initial file loaded is '{}'", filename))
                .hint("rename existing file or import it");
            return (None, crate::util::date::Interval::Empty);
        }
    };
    let data = parse::extract(filename, errs, &contents);
    if errs.is_fatal() {
        return (None, crate::util::date::Interval::Empty);
    }
    let (pairs, period) =
        template::instanciate(filename, errs, data, std::collections::HashMap::new());
    if errs.is_fatal() {
        (None, period)
    } else {
        (Some(pairs), period)
    }
}
