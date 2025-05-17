use std::env::{VarError, args, var};
use std::process::exit;

const OPS: &[&str] = &["+=", "?=", "="];
const OPTS: &[&str] = &[
    "--format-override",
    "-fo",
    "--format-append",
    "-fa",
    "--format",
    "-f",
];

fn main() {
    let args = args().collect::<Vec<String>>();

    let mut format_override = "";
    let mut format_append = "";

    for (i, arg) in args.iter().enumerate() {
        match arg.as_str() {
            "--help" => {
                println!("help is wip");
                exit(0)
            }
            arg if OPTS.contains(&arg) && i + 1 >= args.len() => {
                let arg = args.get(i - 1);

                match arg {
                    None => {
                        eprintln!("not enough arguments");
                    }
                    Some(arg) => {
                        eprintln!("not enough arguments: {arg}");
                    }
                }
                exit(1)
            }
            "--format-override" | "-fo" => {
                format_override = args[i + 1].as_str();
            }
            "--format-append" | "-fa" => {
                format_append = args[i + 1].as_str();
            }
            "--format" | "-f" => match args[i + 1].as_str() {
                "sh" | "bash" | "zsh" => {
                    format_override = "export {N}={V}";
                    format_append = "export {N}={V}:${N}";
                }
                "fish" => {
                    format_override = "set -x {N} {V}";
                    format_append = "set -x {N} {V} ${N}";
                }
                arg => {
                    eprintln!("unrecognized preset format: {arg}");
                    exit(1)
                }
            },
            "--" => {
                break;
            }
            _ => {}
        }
    }

    if format_override.is_empty() {
        eprintln!("nothing for format override");
        exit(1);
    }

    if format_append.is_empty() {
        eprintln!("nothing for format append");
        exit(1);
    }

    let args = args
        .iter()
        .skip(1)
        .flat_map(|s| s.split_whitespace())
        .flat_map(|s| {
            OPS.iter()
                .find_map(|&op| s.split_once(op).map(|(a, b)| [a, op, b]))
                .unwrap_or([s, "", ""])
        })
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>();

    for (i, &arg) in args.iter().enumerate() {
        if OPS.contains(&arg) && (i == 0 || i + 1 >= args.len()) {
            let arg = args.get(i - 1);

            match arg {
                None => {
                    eprintln!("not enough arguments");
                }
                Some(arg) => {
                    eprintln!("not enough arguments: {arg}");
                }
            }

            exit(1)
        }

        if OPS.contains(&arg) {
            let name = args[i - 1];
            let value = args[i + 1];

            parse_op(format_override, format_append, arg, name, value);
        }
    }
}

fn parse_format(format: &str, name: &str, value: &str, current: &str) -> String {
    format
        .replace("{N}", name)
        .replace("{V}", value)
        .replace("{C}", current)
}

fn parse_append(format: &str, name: &str, value: &str, current: &str) {
    if current.split(':').any(|s| s == value) {
        return;
    }

    let result = parse_format(format, name, value, current);

    println!("{result}")
}

fn parse_op(format_override: &str, format_append: &str, op: &str, name: &str, value: &str) {
    match (op, var(name)) {
        ("+=", Ok(current)) => {
            let values = value.split(":");

            for value in values {
                parse_append(format_append, name, value, &current);
            }
        }
        ("+=", Err(VarError::NotPresent)) => {
            let result = parse_format(format_override, name, value, "");

            println!("{result}")
        }
        ("?=", Err(VarError::NotPresent)) => {
            let result = parse_format(format_override, name, value, "");

            println!("{result}")
        }
        ("=", Ok(current)) => {
            let result = parse_format(format_override, name, value, &current);

            println!("{result}")
        }
        ("=", _) => {
            let result = parse_format(format_override, name, value, "");

            println!("{result}")
        }
        (op, _) => {
            unreachable!("Unknown op: {op}")
        }
    }
}
