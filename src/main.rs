mod lib;
mod load;
mod cli;

use lib::{
    date::{Date, Month, Period},
    entry::Duration,
    summary::Calendar,
};
use cli::{
    table::Table,
};

fn main() {
    let filename = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "../expenses.bil".to_string());

    let (entries, errs) = read_entries(&filename);
    println!("{}", errs);
    if let Some(lst) = entries {
        let period = Period(
            Date::from(2020, Month::Sep, 1).unwrap(),
            Date::from(2021, Month::Mar, 1).unwrap(),
        );
        let mut calendar = Calendar::from_spacing(period, Duration::Month, 1);
        calendar.register(&lst);
        let table = Table::from(calendar.contents());
        println!("{}", table);
    }
}

fn read_entries(filename: &str) -> (Option<Vec<lib::entry::Entry>>, load::error::Record) {
    let contents = std::fs::read_to_string(&filename).expect("File not found");
    let mut errs = load::error::Record::new();
    let data = load::parse::extract(&filename, &mut errs, &contents);
    if errs.is_fatal() {
        return (None, errs);
    }
    let pairs = load::template::instanciate(&mut errs, data);
    if errs.is_fatal() {
        (None, errs)
    } else {
        (Some(pairs), errs)
    }
}
