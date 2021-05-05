pub mod error;
pub mod parse;
pub mod template;


pub fn read_entries(filename: &str) -> (Option<Vec<crate::lib::entry::Entry>>, error::Record<parse::Rule>) {
    let contents = std::fs::read_to_string(&filename).expect("File not found");
    let mut errs = error::Record::new();
    let data = parse::extract(&filename, &mut errs, &contents);
    if errs.is_fatal() {
        return (None, errs);
    }
    let pairs = template::instanciate(&mut errs, data);
    if errs.is_fatal() {
        (None, errs)
    } else {
        (Some(pairs), errs)
    }
}
