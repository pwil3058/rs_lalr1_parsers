use clap::crate_authors;
use structopt::StructOpt;

use std::{
    fs,
    io::prelude::*,
    path::{Path, PathBuf},
};

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
    name = "alapgen",
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
    let _expected_number_of_conflicts = if let Some(number) = cl_options.expect {
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
    let mut specification = String::new();
    if let Err(error) = file.read_to_string(&mut specification) {
        writeln!(
            std::io::stderr(),
            "Error reading specification file: {:?}",
            error
        )
        .unwrap();
        std::process::exit(2);
    };
}
