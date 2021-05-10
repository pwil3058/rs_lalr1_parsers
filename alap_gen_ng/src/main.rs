// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use clap::crate_authors;
use structopt::StructOpt;

use std::{
    convert::TryFrom,
    fs,
    io::prelude::*,
    path::{Path, PathBuf},
};

mod alap_gen_ng;
mod attributes;
mod grammar;
mod production;
mod state;
mod symbol;

fn with_changed_extension(path: &Path, new_extension: &str) -> PathBuf {
    let mut new_path = PathBuf::new();
    if let Some(dir) = path.parent() {
        new_path.push(dir);
    };
    new_path.push(path.file_stem().unwrap());
    new_path.set_extension(new_extension);
    new_path
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "alap_gen_ng",
    about = "Augmented Lexical Analyzer and Parser Generator",
    author = crate_authors!(),
)]
struct CLOptions {
    /// Overwrite the output files (if they exist)
    #[structopt(short, long)]
    force: bool,
    /// Total number of shift/reduce and/or reduce/reduce conflicts that are expected.
    #[structopt(short, long)]
    expect: Option<usize>,
    /// The path of the file containing the grammar specification.
    #[structopt(parse(from_os_str))]
    specification: PathBuf,
}

fn main() {
    let cl_options = CLOptions::from_args();
    let output_path = with_changed_extension(&cl_options.specification, "rs");
    if output_path.exists() && !cl_options.force {
        writeln!(
            std::io::stderr(),
            "{}: output file already exists",
            output_path.to_string_lossy()
        )
        .unwrap();
        std::process::exit(1);
    }
    let expected_number_of_conflicts = if let Some(number) = cl_options.expect {
        number
    } else {
        0
    };
    let mut file = match fs::File::open(&cl_options.specification) {
        Ok(file) => file,
        Err(error) => {
            writeln!(
                std::io::stderr(),
                "Error opening specification file: {:?}",
                error
            )
            .unwrap();
            std::process::exit(2);
        }
    };
    let mut specification_text = String::new();
    if let Err(error) = file.read_to_string(&mut specification_text) {
        writeln!(
            std::io::stderr(),
            "Error reading specification file: {:?}",
            error
        )
        .unwrap();
        std::process::exit(2);
    };

    let specification = match grammar::Specification::new(
        specification_text,
        cl_options.specification.to_string_lossy().to_string(),
    ) {
        Ok(spec) => spec,
        Err(error) => {
            writeln!(std::io::stderr(), "Parse failed: {:?}", error).unwrap();
            std::process::exit(2);
        }
    };

    let grammar = match grammar::Grammar::try_from(specification) {
        Ok(grammar) => grammar,
        Err(err) => match err {
            grammar::Error::TooManyErrors(count) => {
                writeln!(std::io::stderr(), "Too many errors: {:?}.", count).unwrap();
                std::process::exit(4);
            }
            grammar::Error::UndefinedSymbols(count) => {
                writeln!(std::io::stderr(), "Undefined symbols: {:?}.", count).unwrap();
                std::process::exit(4);
            }
        },
    };

    if grammar.total_unresolved_conflicts() != expected_number_of_conflicts {
        writeln!(
            std::io::stderr(),
            "Unexpected conflicts ({}) aborting",
            grammar.total_unresolved_conflicts()
        )
        .unwrap();
        std::process::exit(5);
    }
}
