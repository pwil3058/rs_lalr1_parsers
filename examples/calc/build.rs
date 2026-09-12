// Copyright (c) 2026 Peter Williams <pwil3058@bigpond.net.au> <pwil3058@gmail.com>.
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=src/calc.alaps");
    println!("cargo:rerun-if-changed=../../target/debug/alalr1_gen");
    match Command::new("../../target/debug/alalr1_gen")
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
