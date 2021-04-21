mod lib;
use lib::extract;

fn main() {
    let filename = "data.bil";
    let contents = std::fs::read_to_string(filename).expect("File not found");

    match extract::parse::extract(&contents) {
        Ok(data) => println!("{:?}", data),
        Err(err) => println!("{}", err),
    }
}
