use clap::{Arg, ArgAction, ArgMatches, Command};
use failure::{Error, format_err};
use std::collections::HashMap;
use std::error::Error as _;
use std::{env, process};
use tera::{Context as TeraContext, Tera};

fn command_line() -> ArgMatches {
    Command::new("teraform")
        .version("0.1.0")
        .author("Zsombor Gegesy <gzsombor@gmail.com>")
        .about("Templating engine for the command line")
        .arg(
            Arg::new("template")
                .short('t')
                .long("template")
                .value_name("FILE")
                .required(true)
                .help("Template file to use"),
        )
        .arg(
            Arg::new("param")
                .long("param")
                .short('p')
                .help("Sets a parameter for the template in a key=value format")
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new("json")
                .long("json")
                .short('j')
                .help("Loads parameters from a json file")
                .value_name("FILE")
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new("yaml")
                .long("yaml")
                .short('y')
                .help("Loads parameters from a yaml file")
                .value_name("FILE")
                .action(ArgAction::Append),
        )
        .get_matches()
}

fn add_parameters(ctx: &mut TeraContext, cmd_line: &ArgMatches) {
    if let Some(values) = cmd_line.get_many::<String>("param") {
        values.for_each(|kv| {
            let sep_idx = kv.find('=');
            match sep_idx {
                Some(position) => {
                    let (key, value) = kv.split_at(position);
                    ctx.insert(key, &value[1..]);
                }
                None => {
                    ctx.insert(kv.as_str(), &true);
                }
            }
        });
    }
}

fn add_environment(ctx: &mut TeraContext) {
    let mut env_variables = HashMap::new();
    for (key, value) in env::vars() {
        // println!("{}: {}", key, value);
        env_variables.insert(key, value);
    }
    ctx.insert("env", &env_variables);
}

fn run_app() -> Result<String, Error> {
    let matches = command_line();
    let template_path = matches.get_one::<String>("template").unwrap();

    let mut tera = Tera::default();
    tera.add_template_file(template_path, Some("main")).unwrap();
    let mut context = TeraContext::new();
    add_environment(&mut context);
    add_parameters(&mut context, &matches);

    let result = tera.render("main", &context);
    match result {
        Ok(text) => Ok(text),
        Err(template_error) => match template_error.source() {
            Some(cause) => Err(format_err!(
                "Rendering '{}' fails because: {}",
                template_path,
                cause
            )),
            None => Err(format_err!(
                "Rendering '{}' failure: {}",
                template_path,
                template_error
            )),
        },
    }
}

fn main() {
    let result = run_app();
    match result {
        Ok(text) => println!("{}", text),
        Err(msg) => {
            eprintln!("Error: {}", msg);
            process::exit(100);
        }
    }
}
