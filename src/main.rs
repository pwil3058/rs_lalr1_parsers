#[macro_use]
extern crate lazy_static;

mod attributes;
mod bootstrap;
mod grammar;
mod symbols;

fn main() {
    let _parser_specification = grammar::ParserSpecification::new();
    println!("Hello, world!");
}
