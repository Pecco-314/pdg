#![feature(label_break_value)]
mod details;
mod parser;
mod random;
mod token;
use details::With;
use parser::{config, file_range, token};
use simple_combinators::Parser;
use std::{
    env, fs, io,
    ops::Range,
    path::{Path, PathBuf},
};
use token::{cul_token, Config, Token};

fn pause() {
    println!("Finished!");
    println!("(Press any key to exit)");
    io::stdin().read_line(&mut String::new()).unwrap();
}

fn parse_once(buf: &mut &str, is_first: bool) -> (Range<usize>, Vec<Token>, bool) {
    // 解析一个文件标注和其对应的模板
    let range = file_range().parse(buf);
    let mut end = false;
    let range = match range {
        Ok(r) => r,
        Err(_) => {
            end = true;
            if is_first {
                1..11
            } else {
                1..1
            }
        } // 如果尚未生成过，则默认生成1.in~10.in
    };
    let tokens = token().iter(buf).collect();
    (range, tokens, end)
}

fn parse_and_generate(mut buf: &str, fold: PathBuf) {
    let mut is_first = true;
    loop {
        let (range, tokens, end) = parse_once(&mut buf, is_first);
        is_first = false;
        for i in range {
            generate(i, &tokens, &fold);
        }
        if end {
            break;
        }
    }
}

fn generate(fileid: usize, tokens: &Vec<Token>, fold: &PathBuf) {
    println!("Generating {}.in", fileid);
    let target = fold.join(&fileid.to_string().with_str(".in"));
    let gens = cul_token(&tokens).expect("Something went wrong while culculating tokens");
    let mut s = String::new();
    for i in gens.iter() {
        s.push_str(
            &i.generate_str()
                .expect("Something went wrong while generating random numbers"),
        );
    }
    fs::write(target, s).expect("Failed to write");
}

fn get_template<'a>() -> (PathBuf, String) {
    let args: Vec<String> = env::args().collect();
    let path = if args.len() > 1 {
        &args[1]
    } else {
        "template.txt" // 默认路径
    };
    (
        Path::new(path).to_path_buf(),
        fs::read_to_string(path).expect("Failed to read"),
    )
}

fn get_fold(template: &PathBuf, config: &Config) -> PathBuf {
    let fold = if let Some(s) = &config.fold {
        Path::new(&s).to_path_buf() // 如果重定向了输出文件夹则应用
    } else {
        template
            .parent()
            .expect("invaild path format")
            .join("testdata")
    };
    println!("{:?}", fold);
    fs::create_dir_all(&fold).expect("Failed to create directory");
    fold
}

fn main() {
    let (path, template) = get_template();
    let mut buf = template.as_str();
    let config = config().parse(&mut buf).unwrap(); // 解析配置
    let fold = get_fold(&path, &config);
    parse_and_generate(buf, fold);
    pause();
}
