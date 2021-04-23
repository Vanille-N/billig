mod lib;

fn main() {
    let filename = std::env::args().nth(1).unwrap_or_else(|| "../expenses.bil".to_string());

    let (entries, errs) = read_entries(&filename);
    println!("{}", errs);
    println!("{:?}", entries);

}

fn read_entries(filename: &str) -> (Option<Vec<(lib::date::Date, lib::entry::Entry)>>, lib::error::ErrorRecord) {
    let contents = std::fs::read_to_string(&filename).expect("File not found");
    let mut errs = lib::error::ErrorRecord::new();
    let data = lib::parse::extract(&filename, &mut errs, &contents);
    if errs.is_fatal() {
        return (None, errs);
    }
    let pairs = lib::template::instanciate(&mut errs, data);  
    if errs.is_fatal()  {
        (None, errs)
    } else {
        (Some(pairs), errs)
    }
}
