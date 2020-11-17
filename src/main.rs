mod details;
mod parser;
mod token;
use parser::token;
use simple_combinators::Parser;
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    let path = if args.len() > 1 {
        &args[1]
    } else {
        "template.txt"
    };
    let parent = Path::new(path).parent().expect("invaild path format");

    let template = fs::read_to_string(path).expect("Failed to read");
    let mut buf = template.as_str();
    let token = token();
    let iter = token.iter(&mut buf).map(|x| x.generate());
    let s: String = iter.collect();

    let path = parent.join("data");
    fs::create_dir(&path).expect("Failed to create directory");
    fs::write(path.join("1.in"), s).expect("Failed to write");
}
