// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::process::Command;

use lalr1_lib::ParserGenerator;

fn main() {
    println!("cargo:rerun-if-changed=src/calc_no_aug.laps");
    match ParserGenerator::new("src/calc_no_aug.laps") {
        Ok(gen) => match gen.write_parser_code_to_file("src/calc_no_aug.rs") {
            Ok(_) => {
                Command::new("rustfmt")
                    .args(&["src/calc_no_aug.rs"])
                    .status()
                    .expect("prebuild: cargo run rustfmt failed");
            }
            Err(e) => panic!("failed prebuild: {}", e),
        },
        Err(e) => panic!("Build error{}", e),
    }
    println!("cargo:rerun-if-changed=build.rs");
}
