// Copyright (c) 2026 Peter Williams <pwil3058@bigpond.net.au> <pwil3058@gmail.com>.
use std::{path::Path, process::Command};

fn main() {
    let lalr1_gen_path = "../target/debug/lalr1_gen";
    if Path::new(lalr1_gen_path).exists() {
        match Command::new(lalr1_gen_path)
            .args(["-f", "src/parser.laps"])
            .status()
        {
            Ok(status) => {
                if status.success() {
                    Command::new("rustfmt")
                        .args(["src/parser.rs"])
                        .status()
                        .unwrap();
                } else {
                    panic!("failed prebuild formatting: {status}");
                };
            }
            Err(err) => panic!("Build error: {err}"),
        }
        println!("cargo::rerun-if-changed=build");
    }
}
