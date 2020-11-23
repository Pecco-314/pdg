#![feature(label_break_value)]
#![feature(bool_to_option)]
mod details;
mod parser;
mod random;
mod token;
use crate::{
    details::{error_info, Ignore},
    parser::{config, file_range, token},
    token::{cul_token, Config, Token},
};
use colour::*;
use powershell_script;
use simple_combinators::{ParseError, Parser};
use std::{
    env, fs, io,
    io::ErrorKind,
    ops::Range,
    path::{Path, PathBuf},
    process::exit,
};

fn pause() {
    println!("(Press Enter/Return to exit)");
    io::stdin().read_line(&mut String::new()).ignore();
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
    let results = token().iter(buf).with_result().collect();
    let tokens = handle_parse_result(results);
    (range, tokens, end)
}

fn handle_parse_result(results: Vec<Result<Token, ParseError>>) -> Vec<Token> {
    let err = results.last().ignore().as_ref().err().ignore(); // 最后一项永远是错误
    if !err.position.is_empty() {
        // pos若空则说明已到EOF // <- FIXIT:这样判断是不好的
        e_red!("error");
        eprint!(": Something went wrong while parsing ",);
        e_white_ln!("\"{}\"", err.position);
        exit(1);
    }
    results[..results.len() - 1]
        .iter()
        .map(|r| r.as_ref().unwrap().clone())
        .collect()
}

fn parse_and_generate(mut buf: &str, folder: PathBuf, config: &Config) {
    let mut is_first = true;
    loop {
        let (range, tokens, end) = parse_once(&mut buf, is_first);
        is_first = false;
        for i in range {
            generate(i, &tokens, &folder, config);
        }
        if end {
            break;
        }
    }
    println!("Finished!");
}

fn generate(fileid: usize, tokens: &Vec<Token>, folder: &PathBuf, config: &Config) {
    let prefix = if let Some(prefix) = &config.prefix {
        prefix
    } else {
        ""
    };
    let filename = format!("{}{}.in", prefix, fileid);
    println!("Generating {}", filename);
    let target = folder.join(&filename);
    let gens = cul_token(&tokens)
        .unwrap_or_else(|| error_info("Something went wrong while culculating tokens"));
    let mut s = String::new();
    for i in gens.iter() {
        s.push_str(&i.generate_str().unwrap_or_else(|| {
            error_info(&format!(
                "Something went wrong while generating {}",
                filename
            ))
        }));
    }
    fs::write(&target, s).unwrap_or_else(|err| match err {
        _ if err.kind() == ErrorKind::PermissionDenied => {
            error_info("Permission denied while trying to write generated results to the file")
        }
        _ => error_info(
            "Some unknown error occurred while trying to write generated results to the file",
        ),
    });
    if let Some(std) = &config.std {
        let output = format!("{}{}.out", prefix, fileid);
        let input = &filename;
        run_std(&folder, &output, &input, std);
    }
}
fn run_std(folder: &PathBuf, output: &str, input: &str, std: &str) {
    println!("Generating {}", output);
    powershell_script::run(
        &format!(
            "Get-Content {} | {} | Out-File {}",
            folder.join(input).to_str().ignore(),
            std,
            folder.join(output).to_str().ignore(),
        ),
        false,
    )
    .unwrap_or_else(|_| error_info(&format!("Something went wrong while generating {}", input)));
}

fn get_template<'a>() -> (PathBuf, String) {
    let args: Vec<String> = env::args().collect();
    let path = if args.len() > 1 {
        &args[1]
    } else {
        "template.txt" // 默认路径
    };
    (Path::new(path).to_path_buf(), {
        // TODO: CRLF->LF
        let read_result = fs::read_to_string(path);
        match read_result {
            Ok(result) => result,
            Err(e) if e.kind() == ErrorKind::NotFound => {
                error_info("Cannot find the template file");
            }
            Err(e) if e.kind() == ErrorKind::PermissionDenied => {
                error_info("Permission denied for reading the template file");
            }
            Err(e) if e.kind() == ErrorKind::InvalidData => {
                error_info("The template file was not valid UTF-8 file");
            }
            Err(_) => {
                error_info("Some unknown error occurred while reading the template file");
            }
        }
    })
}

fn get_folder(template: &PathBuf, config: &Config) -> PathBuf {
    let folder = if let Some(s) = &config.folder {
        Path::new(&s).to_path_buf() // 如果重定向了输出文件夹则应用
    } else {
        template.parent().ignore().join("testdata")
    };
    match fs::create_dir_all(&folder) {
        Ok(()) => folder,
        Err(e) if e.kind() == ErrorKind::PermissionDenied => error_info(
            "The target folder did not exist, but the permission denied for creating it.",
        ),
        Err(_) => error_info(
            "Some unknown error occurred while creating the target folder. Maybe the directory format was wrong.",
        ),
    }
}

fn main() {
    let (path, template) = get_template();
    let mut buf = template.as_str();
    let config = config().parse(&mut buf).ignore(); // 解析配置
    let folder = get_folder(&path, &config);
    parse_and_generate(buf, folder, &config);
    if let Some(true) | None = config.pause {
        pause();
    }
}
