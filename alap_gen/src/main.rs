// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use clap::crate_authors;
use structopt::StructOpt;

use std::{
    convert::TryFrom,
    fs,
    io::prelude::*,
    path::{Path, PathBuf},
};

#[cfg(not(feature = "bootstrap"))]
mod alap_gen;
mod attributes;
#[cfg(feature = "bootstrap")]
mod bootstrap;
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
    /// Overwrite the output files (if they exist).
    #[structopt(short, long)]
    force: bool,
    /// Don't fail if shift/reduce conflicts even if differ from expected.
    #[structopt(long)]
    ignore_sr_conflicts: bool,
    /// Don't fail if reduce/reduce conflicts even if differ from expected.
    #[structopt(long)]
    ignore_rr_conflicts: bool,
    /// Specify the path of the required output file (if different to the default).
    #[structopt(short, long)]
    output: Option<PathBuf>,
    /// The path of the file containing the grammar specification.
    #[structopt(parse(from_os_str))]
    specification: PathBuf,
}

fn main() {
    let cl_options = CLOptions::from_args();
    let output_path = if let Some(output_path) = cl_options.output {
        output_path
    } else {
        with_changed_extension(&cl_options.specification, "rs")
    };
    if output_path.exists() && !cl_options.force {
        eprintln!(
            "{}: output file already exists",
            output_path.to_string_lossy()
        );
        std::process::exit(1);
    }
    let mut file = match fs::File::open(&cl_options.specification) {
        Ok(file) => file,
        Err(error) => {
            eprintln!("Error opening specification file: {error:?}");
            std::process::exit(2);
        }
    };
    let mut specification_text = String::new();
    if let Err(error) = file.read_to_string(&mut specification_text) {
        eprintln!("Error reading specification file: {error:?}");
        std::process::exit(2);
    };

    let specification = match grammar::Specification::new(
        &specification_text,
        &cl_options.specification.to_string_lossy().to_string(),
    ) {
        Ok(spec) => spec,
        Err(error) => {
            eprintln!("Parse failed: {error:?}");
            std::process::exit(2);
        }
    };

    let grammar = match grammar::Grammar::try_from((
        specification,
        cl_options.ignore_sr_conflicts,
        cl_options.ignore_rr_conflicts,
    )) {
        Ok(grammar) => grammar,
        Err(err) => {
            match err {
                grammar::Error::TooManyErrors(count) => {
                    eprintln!("Too many errors: {count:?}.");
                    std::process::exit(4);
                }
                grammar::Error::UndefinedSymbols(count) => {
                    eprintln!("Undefined symbols: {count:?}.");
                    std::process::exit(4);
                }
                grammar::Error::UnexpectedSRConflicts(count, expected, report) => {
                    eprintln!("{report}\nUnexpected shift/reduce conflicts: {count} expected: {expected}.");
                    std::process::exit(4);
                }
                grammar::Error::UnexpectedRRConflicts(count, expected, report) => {
                    eprintln!("{report}\nUnexpected reduce/reduce conflicts: {count} expected: {expected}.");
                    std::process::exit(4);
                }
            }
        }
    };

    if let Err(err) = grammar.write_parser_code_to_file(&output_path) {
        eprintln!(
            "{}: problems writing file: {:?}.",
            output_path.to_string_lossy(),
            err
        );
        std::process::exit(6);
    }

    let description_file = with_changed_extension(&output_path, "states");
    if let Err(err) = grammar.write_description(&description_file) {
        eprintln!(
            "{}: problems writing file: {:?}.",
            output_path.to_string_lossy(),
            err
        );
        std::process::exit(7);
    };
}
