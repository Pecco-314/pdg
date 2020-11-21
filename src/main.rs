#![feature(label_break_value)]
mod details;
mod parser;
mod random;
mod token;
use crate::{
    details::error_info,
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
            e_red!("error");
            eprint!(": Something went wrong while parsing ",);
            e_white_ln!("\"...{}...\"", pos);
            exit(1);
        }
        _ => {}
    }
    (range, tokens, end)
}

fn parse_and_generate(mut buf: &str, folder: PathBuf, config: &Config) {
    let mut is_first = true;
    loop {
        let (range, tokens, end) = parse_once(&mut buf, is_first);
        // println!("{:?}", tokens);
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
            folder.join(input).to_str().unwrap(),
            std,
            folder.join(output).to_str().unwrap(), // 这里应该不会panic
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
        template.parent().unwrap().join("testdata") // 此处不会panic
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
    let config = config().parse(&mut buf).unwrap(); // 解析配置，此处不会panic
    let folder = get_folder(&path, &config);
    parse_and_generate(buf, folder, &config);
    if let Some(true) | None = config.pause {
        pause();
    }
}
