#![feature(iterator_fold_self)]
#![feature(bool_to_option)]
#![feature(label_break_value)]
mod details;
mod parser;
mod random;
mod token;
use crate::{
    details::{error_info, Ignore},
    parser::{config, file_range, token},
    token::{Config, Token},
};
use colour::*;
use powershell_script;
use simple_combinators::{
    combinator::{attempt, preview},
    parser::string,
    ParseError, Parser,
};
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
    let results = attempt(token()).iter(buf).with_result().collect();
    let tokens = handle_parse_result(results);
    (range, tokens, end)
}

fn handle_parse_result(results: Vec<Result<Token, ParseError>>) -> Vec<Token> {
    let err = results.last().ignore().as_ref().err().ignore(); // 最后一项永远是错误
    let mut position = err.position;
    if !err.position.is_empty() && preview(string(":>")).parse(&mut position).is_err() {
        // pos不为空，且接下来无文件标注，则说明发生了错误
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
    let mut s = String::new();
    for i in tokens.iter() {
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
    let std_path = Path::new(std);
    let std_parent = std_path
        .parent()
        .unwrap_or_else(|| error_info("Incorrect standard program path"));
    let mut std_path = String::new();
    if std_parent.parent().is_none() {
        std_path.push_str("./") // 如果是单独的文件名，加一个./ （这段有没有更好的写法？）
    };
    std_path.push_str(std);
    let script = &format!(
        "Get-Content {} | {} | Out-File {} -Encoding default",
        folder.join(input).to_str().ignore(),
        std_path,
        folder.join(output).to_str().ignore(),
    );
    powershell_script::run(script, false).unwrap_or_else(|_| {
        error_info(&format!(
            "Something went wrong while generating {}\nPlease check your standard program and the powershell script: {}",
            input, script
        ))
    });
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
