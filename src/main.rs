mod details;
mod parser;
mod random;
mod token;
use parser::token;
use simple_combinators::Parser;
use std::env;
use std::fs;
use std::path::Path;
use token::{cul_token, Token};

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
    let tokens: Vec<Token> = token().iter(&mut buf).collect();
    let gens = cul_token(&tokens).expect("Something went wrong while culculating tokens");
    let mut s = String::new();
    for i in gens.iter() {
        s.push_str(
            &i.generate()
                .expect("Something went wrong while generating random numbers"),
        );
    }

    let path = parent.join("data");
    fs::create_dir_all(&path).expect("Failed to create directory");
    fs::write(path.join("1.in"), s).expect("Failed to write");
}
