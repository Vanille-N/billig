mod lib;
use lib::extract;

fn main() {
    match extract::parse::extract("data.bil") {
        Ok(data) => println!("{:?}", data),
        Err(err) => println!("{}", err),
    }
}
