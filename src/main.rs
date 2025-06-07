use std::collections::HashSet;
use std::env::{VarError, args, var};
use std::io::{IsTerminal, Read, stdin};
use std::process::exit;

const HELP: &str = include_str!("help");
const OPS: &[&str] = &["+=!", "+=", "=!", "="];
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

    if args.is_empty() {
        println!("{HELP}");
        exit(0)
    }

    let mut format_override = "";
    let mut format_append = "";
    let mut force = false;

    for (i, arg) in args.iter().enumerate() {
        match arg.as_str() {
            "--help" => {
                println!("{HELP}");
                exit(0);
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
            "--force" | "-F" => force = true,
            "--format-override" | "-fo" => format_override = args[i + 1].as_str(),
            "--format-append" | "-fa" => format_append = args[i + 1].as_str(),
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

    let input = input();
    let input = input
        .iter()
        .flat_map(|s| s.lines())
        .filter(|s| !s.starts_with("//"))
        .filter(|s| !s.starts_with("#"));

    let args = args
        .iter()
        .skip(1)
        .map(|s| s.as_str())
        .chain(input)
        .flat_map(|s| s.split_whitespace())
        .flat_map(|s| {
            OPS.iter()
                .find_map(|&op| s.split_once(op).map(|(a, b)| [a, op, b]))
                .unwrap_or([s, "", ""])
        })
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>();

    let mut dedup = HashSet::new();

    for (i, &op) in args.iter().enumerate() {
        if OPS.contains(&op) && (i == 0 || i + 1 >= args.len()) {
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

        if OPS.contains(&op) {
            let name = args[i - 1];
            let value = args[i + 1];

            if !force && dedup.contains(&(name, op, value)) {
                continue;
            }

            if op == "+=" || op == "=" {
                dedup.insert((name, op, value));
            }

            let value = parse_op(force, format_override, format_append, op, name, value);

            for set in value.iter().flatten() {
                println!("{set}")
            }
        }
    }
}

fn input() -> Option<String> {
    let mut str = String::new();
    let mut stdin = stdin();

    if stdin.is_terminal() {
        return None;
    }

    let res = stdin.read_to_string(&mut str);

    match res {
        Ok(_) => Some(str),
        Err(_) => None,
    }
}

fn parse_format(format: &str, name: &str, value: &str, current: &str) -> String {
    format
        .replace("{N}", name)
        .replace("{V}", value)
        .replace("{C}", current)
}

fn parse_append(
    force: bool,
    format: &str,
    name: &str,
    value: &str,
    current: &str,
) -> Option<String> {
    if !force && current.split(':').any(|s| s == value) {
        return None;
    }

    let result = parse_format(format, name, value, current);

    Some(result)
}

fn parse_op(
    force: bool,
    format_override: &str,
    format_append: &str,
    op: &str,
    name: &str,
    value: &str,
) -> Option<Vec<String>> {
    match (op, var(name)) {
        ("+=!", Ok(current)) => {
            let values = value.split(":");

            let result = values
                .flat_map(|s| parse_append(true, format_append, name, s, &current))
                .collect();

            Some(result)
        }
        ("+=!", Err(VarError::NotPresent)) => {
            let result = parse_format(format_override, name, value, "");

            Some(vec![result])
        }
        ("+=", Ok(current)) => {
            let values = value.split(":");

            let result = values
                .flat_map(|s| parse_append(force, format_append, name, s, &current))
                .collect();

            Some(result)
        }
        ("+=", Err(VarError::NotPresent)) => {
            let result = parse_format(format_override, name, value, "");

            Some(vec![result])
        }
        ("=!", Ok(current)) => {
            let result = parse_format(format_override, name, value, &current);

            Some(vec![result])
        }
        ("=!", Err(VarError::NotPresent)) => {
            let result = parse_format(format_override, name, value, "");

            Some(vec![result])
        }
        ("=", _) if force => {
            let result = parse_format(format_override, name, value, "");

            Some(vec![result])
        }
        ("=", Err(VarError::NotPresent)) => {
            let result = parse_format(format_override, name, value, "");

            Some(vec![result])
        }
        ("=", Ok(_)) => None,
        (op, _) => {
            unreachable!("Unknown op: {op}")
        }
    }
}
