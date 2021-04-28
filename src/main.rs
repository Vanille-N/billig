mod lib;
mod load;

use lib::{
    date::{Date, Month, Period},
    entry::{Span, Duration, Window},
    summary::Summary,
};

fn main() {
    let filename = std::env::args().nth(1).unwrap_or_else(|| "../expenses.bil".to_string());

    let (entries, errs) = read_entries(&filename);
    println!("{}", errs);
    if let Some(lst) = entries {
        let period = (Date::from(2020, Month::Dec, 12).unwrap(), Date::from(2021, Month::Apr, 5).unwrap());
        let whole = Span::from(Duration::Year, Window::Posterior, 1).period(Date::from(2020, Month::Sep, 1).unwrap());
        let mut summary = Summary::new_period(period);
        let mut whole = Summary::new_period(whole);
        for entry in lst {
            summary += &entry;
            whole += &entry;
            if let Some(e) = entry.intersect(period) {
                println!("{}", e);
            }
        }
        println!("{:?}", summary);
        println!("{:?}", whole);
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
    if errs.is_fatal()  {
        (None, errs)
    } else {
        (Some(pairs), errs)
    }
}
