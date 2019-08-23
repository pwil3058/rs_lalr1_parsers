#[macro_use]
extern crate lazy_static;
extern crate clap;
extern crate regex;

#[macro_export]
macro_rules! impl_ident_cmp {
    ( $ident:ident ) => {
        impl std::cmp::PartialEq for $ident {
            fn eq(&self, other: &Self) -> bool {
                self.ident == other.ident
            }
        }

        impl std::cmp::Eq for $ident {}

        impl std::cmp::Ord for $ident {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.ident.cmp(&other.ident)
            }
        }

        impl std::cmp::PartialOrd for $ident {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }
    };
}

use std::{
    fs,
    io::prelude::*,
    path::{Path, PathBuf},
};

#[cfg(not(feature = "bootstrap"))]
mod alapgen;
mod attributes;
#[cfg(feature = "bootstrap")]
mod bootstrap;
mod grammar;
mod state;
mod symbols;

fn with_changed_extension(path: &Path, new_extension: &str) -> PathBuf {
    let mut new_path = PathBuf::new();
    if let Some(dir) = path.parent() {
        new_path.push(dir);
    };
    new_path.push(path.file_stem().unwrap());
    new_path.set_extension(new_extension);
    new_path
}

fn main() {
    let matches = clap::App::new("Augmented Lexical Analyzer and Parser Generator")
        .arg(
            clap::Arg::with_name("force")
                .short("f")
                .long("force")
                .takes_value(false),
        )
        .arg(clap::Arg::with_name("input").required(true))
        .get_matches();
    let force = matches.is_present("force");
    let file_name = matches
        .value_of("input")
        .expect("\"input\" is a required argument");
    let output_path = with_changed_extension(Path::new(file_name), "rs");
    if output_path.exists() && !force {
        writeln!(
            std::io::stderr(),
            "{}: output file already exists",
            output_path.to_string_lossy()
        )
        .unwrap();
        std::process::exit(1);
    }
    let expected_number_of_conflicts = 0;
    let mut file = fs::File::open(file_name).unwrap();
    let mut input = String::new();
    file.read_to_string(&mut input).unwrap();
    let grammar_specification =
        match grammar::GrammarSpecification::new(input, file_name.to_string()) {
            Ok(spec) => spec,
            Err(error) => {
                writeln!(std::io::stderr(), "Parse failed: {:?}", error).unwrap();
                std::process::exit(2);
            }
        };

    for symbol in grammar_specification.symbol_table.unused_symbols() {
        let location = symbol.defined_at().unwrap();
        grammar::report_warning(
            &location,
            &format!("Symbol \"{}\" is not used", symbol.name()),
        );
    }

    let mut undefined_symbols = 0;
    for symbol in grammar_specification.symbol_table.undefined_symbols() {
        for location in symbol.used_at() {
            grammar::report_error(
                &location,
                &format!("Symbol \"{}\" is not defined", symbol.name()),
            );
        }
        undefined_symbols += 1;
    }

    if (undefined_symbols + grammar_specification.error_count) > 0 {
        writeln!(
            std::io::stderr(),
            "Too man errors {} aborting.",
            (undefined_symbols + grammar_specification.error_count)
        )
        .unwrap();
        std::process::exit(3);
    }

    let grammar = match grammar::Grammar::new(grammar_specification) {
        Ok(grammar) => grammar,
        Err(err) => {
            writeln!(std::io::stderr(), "Grammar failed to build: {:?}.", err).unwrap();
            std::process::exit(4);
        }
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

    if let Err(err) = grammar.write_parser_code(&output_path) {
        writeln!(
            std::io::stderr(),
            "{}: problems writing file: {:?}.",
            output_path.to_string_lossy(),
            err
        )
        .unwrap();
        std::process::exit(6);
    }

    let description_file = with_changed_extension(Path::new(file_name), "states");
    if let Err(err) = grammar.write_description(&description_file) {
        writeln!(
            std::io::stderr(),
            "{}: problems writing file: {:?}.",
            output_path.to_string_lossy(),
            err
        )
        .unwrap();
        std::process::exit(7);
    };
}
