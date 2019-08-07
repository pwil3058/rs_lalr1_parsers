#[macro_use]
extern crate lazy_static;
extern crate clap;

use std::fs;
use std::io::prelude::*;

use lalr1plus::parser::*;

mod attributes;
mod bootstrap;
mod grammar;
mod symbols;

fn main() {
    let matches = clap::App::new("Augmented Lexical Analyzer and Parser Generator")
        .arg(clap::Arg::with_name("input").required(true))
        .get_matches();
    let file_name = matches.value_of("input").expect("\"input\" is a required argument");
    let mut file = fs::File::open(file_name).unwrap();
    let mut input = String::new();
    file.read_to_string(&mut input).unwrap();
    let mut parser_specification = grammar::ParserSpecification::new();
    parser_specification.parse_text(input, file_name.to_string());
    println!("Hello, world!");
}
