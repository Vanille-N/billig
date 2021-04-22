mod lib;

fn main() {
    let filename = std::env::args().skip(1).next().unwrap_or("data.bil".to_string());
    let contents = std::fs::read_to_string(&filename).expect("File not found");

    let mut errs = lib::error::ErrorRecord::new();
    let data = {
        let data = lib::parse::extract(&filename, &mut errs, &contents);
        if errs.is_fatal() {
            println!("{}", errs);
            panic!();
        }
        data
    };
    let pairs = {
        let pairs = lib::template::instanciate(&mut errs, data);
        if errs.is_fatal()  {
            println!("{}", errs);
            panic!();
        } else if errs.count_warnings() > 0 {
            println!("{}", errs);
        }
        pairs
    };
    println!("{:#?}", pairs);
}
