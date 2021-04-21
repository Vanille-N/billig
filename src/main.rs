mod lib;
use lib::extract;
use lib::instanciate;

fn main() {
    let filename = "data.bil";
    let contents = std::fs::read_to_string(filename).expect("File not found");

    let data = match extract::parse::extract(&contents) {
        Ok(data) => data,
        Err(err) => {
            println!("{}", err);
            panic!();
        }
    };
    let pairs = match instanciate::instanciate(data) {
        Ok(pairs) => pairs,
        Err(err) => {
            println!("{}", err);
            panic!();
        }
    };
}
