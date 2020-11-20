#![feature(label_break_value)]
mod details;
mod parser;
mod random;
mod token;
use details::With;
use parser::{file_range, token};
use simple_combinators::Parser;
use std::env;
use std::fs;
use std::io;
use std::path::Path;
use token::{cul_token, Token};

fn pause() {
    println!("Finished!");
    println!("(Press any key to exit)");
    io::stdin().read_line(&mut String::new()).unwrap();
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let template_path = if args.len() > 1 {
        &args[1]
    } else {
        "template.txt" // 默认路径
    };
    let parent = Path::new(template_path)
        .parent()
        .expect("invaild path format");
    let template = fs::read_to_string(template_path).expect("Failed to read");
    let parent_path = parent.join("testdata");
    fs::create_dir_all(&parent_path).expect("Failed to create directory");
    let mut buf = template.as_str();

    let mut exit = false;
    let mut is_first = true;
    'out: while !exit {
        let range = file_range().parse(&mut buf);
        let range = match range {
            Ok(r) => r,
            Err(_) => {
                if !is_first {
                    break 'out;
                }
                exit = true;
                1..11
            } // 如果尚未生成过，则默认生成1.in~10.in并退出；否则直接退出
        };
        is_first = false;
        let tokens: Vec<Token> = token().iter(&mut buf).collect();
        for i in range {
            println!("Generating {}.in", i);
            let target_path = parent_path.join(&i.to_string().with_str(".in"));
            let gens = cul_token(&tokens).expect("Something went wrong while culculating tokens");
            let mut s = String::new();
            for i in gens.iter() {
                s.push_str(
                    &i.generate()
                        .expect("Something went wrong while generating random numbers"),
                );
            }
            fs::write(target_path, s).expect("Failed to write");
        }
    }
    pause();
}
