extern crate tera;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate failure;

use clap::{App, Arg, ArgMatches};
use failure::Error;
use std::collections::HashMap;
use std::{env, process};
use tera::{Context as TeraContext, Tera};

fn command_line<'a>() -> ArgMatches<'a> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about(crate_description!())
        .arg(
            Arg::with_name("template")
                .short("t")
                .long("template")
                .value_name("FILE")
                .required(true)
                .help("Template file to use")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("param")
                .long("param")
                .short("p")
                .help("Sets a parameter for the template in a key=value format")
                .multiple(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("json")
                .long("json")
                .short("j")
                .multiple(true)
                .help("Loads parameters from a json file")
                .value_name("FILE")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("yaml")
                .long("yaml")
                .short("y")
                .multiple(true)
                .help("Loads parameters from a yaml file")
                .value_name("FILE")
                .takes_value(true),
        )
        .get_matches()
}

fn add_parameters(ctx: &mut TeraContext, cmd_line: &ArgMatches) {
    if let Some(values) = cmd_line.values_of("param") {
        values.for_each(|kv| {
            let sep_idx = kv.find('=');
            match sep_idx {
                Some(position) => {
                    let (key, value) = kv.split_at(position);
                    ctx.insert(key, &value[1..]);
                }
                None => {
                    ctx.insert(kv, &true);
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
    let template_path = matches.value_of("template").unwrap();

    let mut tera = Tera::default();
    tera.add_template_file(template_path, Some("main")).unwrap();
    let mut context = TeraContext::new();
    add_environment(&mut context);
    add_parameters(&mut context, &matches);

    let result = tera.render("main", &context);
    match result {
        Ok(text) => Ok(text),
        Err(template_error) => match template_error.iter().nth(1) {
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
