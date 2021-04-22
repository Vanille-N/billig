mod lib;

fn main() {
    let filename = std::env::args().skip(1).next().unwrap_or("data.bil".to_string());
    let contents = std::fs::read_to_string(filename).expect("File not found");

    let data = match lib::parse::extract(&contents) {
        Ok(data) => data,
        Err(err) => {
            println!("{}", err);
            panic!();
        }
    };
    let pairs = match lib::template::instanciate(data) {
        Ok(pairs) => pairs,
        Err(err) => {
            println!("{}", err);
            panic!();
        }
    };
    println!("{:#?}", pairs);
}
