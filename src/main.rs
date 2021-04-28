mod lib;
mod load;

fn main() {
    let filename = std::env::args().nth(1).unwrap_or_else(|| "../expenses.bil".to_string());

    let (entries, errs) = read_entries(&filename);
    println!("{}", errs);
    if let Some(lst) = entries {
        use lib::date::{Date, Month};
        for entry in lst {
            if let Some(e) = entry.intersect((Date::from(2020, Month::Dec, 12).unwrap(), Date::from(2021, Month::Apr, 5).unwrap())) {
                println!("{}", e);
            }
        }
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
