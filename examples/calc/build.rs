// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=src/calc.alaps");
    println!("cargo:rerun-if-changed=../../target/debug/alap_gen");
    match Command::new("../../target/debug/alap_gen")
        .args(&["-f", "src/calc.alaps"])
        .status()
    {
        Ok(status) => {
            if status.success() {
                Command::new("rustfmt")
                    .args(&["src/calc.rs"])
                    .status()
                    .unwrap();
            } else {
                panic!("failed prebuild: {}", status);
            };
        }
        Err(err) => panic!("Build error: {}", err),
    }
    println!("cargo:rerun-if-changed=build.rs");
}
