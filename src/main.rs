#![feature(label_break_value)]
mod details;
mod parser;
mod random;
mod token;
use crate::{
    parser::{config, file_range, token},
    token::{cul_token, Config, Token},
};
use powershell_script;
use simple_combinators::{ParseError, Parser};
use std::{
    env, fs, io,
    ops::Range,
    path::{Path, PathBuf},
    process::exit,
};

fn pause() {
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
        } // 没有发现文件标注，如果尚未生成过，则默认生成1.in~10.in
    };
    let tokens = token().iter(buf).collect();
    let next_parse_result = token().parse(buf);
    match next_parse_result {
        Err(ParseError { position: pos }) if !pos.is_empty() => {
            println!("Something went wrong while parsing \"...{}...\"", pos);
            exit(1);
        }
        _ => {}
    }
    (range, tokens, end)
}

fn parse_and_generate(mut buf: &str, fold: PathBuf, config: &Config) {
    let mut is_first = true;
    loop {
        let (range, tokens, end) = parse_once(&mut buf, is_first);
        // println!("{:?}", tokens);
        is_first = false;
        for i in range {
            generate(i, &tokens, &fold, config);
        }
        if end {
            break;
        }
    }
    println!("Finished!");
}

fn generate(fileid: usize, tokens: &Vec<Token>, fold: &PathBuf, config: &Config) {
    let prefix = if let Some(prefix) = &config.prefix {
        prefix
    } else {
        ""
    };
    let filename = format!("{}{}.in", prefix, fileid);
    println!("Generating {}", filename);
    let target = fold.join(filename);
    let gens = cul_token(&tokens).expect("Something went wrong while culculating tokens");
    let mut s = String::new();
    for i in gens.iter() {
        s.push_str(
            &i.generate_str()
                .expect("Something went wrong while generating random numbers"),
        );
    }
    fs::write(&target, s).expect("Failed to write");
    let output = format!("{}{}.out", prefix, fileid);
    if let Some(std) = &config.std {
        run_std(&output, &target, std).unwrap();
    }
}
fn run_std(output: &str, target: &PathBuf, std: &str) -> Option<()> {
    println!("Generating {}", output);
    powershell_script::run(
        &format!(
            "Get-Content {} | {} | Out-File {}",
            target.to_str()?,
            std,
            target.parent()?.join(output).to_str()?,
        ),
        false,
    )
    .ok()?;
    Some(())
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
    fs::create_dir_all(&fold).expect("Failed to create directory");
    fold
}

fn main() {
    let (path, template) = get_template();
    let mut buf = template.as_str();
    let config = config().parse(&mut buf).unwrap(); // 解析配置
    let fold = get_fold(&path, &config);
    parse_and_generate(buf, fold, &config);
    if let Some(true) | None = config.pause {
        pause();
    }
}
