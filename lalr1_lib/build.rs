// Copyright (c) 2026 Peter Williams <pwil3058@bigpond.net.au> <pwil3058@gmail.com>.
use std::{path::Path, process::Command};

fn main() {
    let lalr1_gen_path = "../target/debug/lalr1_gen";
    if Path::new(lalr1_gen_path).exists() {
        println!("cargo::rerun-if-changed=src/parser.laps");
        println!("cargo::rerun-if-changed={lalr1_gen_path}");
        match Command::new(lalr1_gen_path)
            .args(["-f", "src/parser.laps"])
            .status()
        {
            Ok(status) => {
                if status.success() {
                    Command::new("rustfmt")
                        .args(["src/parser"])
                        .status()
                        .unwrap();
                } else {
                    panic!("failed prebuild: {status}");
                };
            }
            Err(err) => panic!("Build error: {err}"),
        }
        println!("cargo::rerun-if-changed=buildx");
    }
}
