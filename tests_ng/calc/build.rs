// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=src/calc.alaps");
    println!("cargo:rerun-if-changed=../../target/debug/alap_gen_ng");
    let status = Command::new("../../target/debug/alap_gen_ng")
        .args(&["-f", "-e1", "src/calc.alaps"])
        .status()
        .unwrap();
    if status.success() {
        Command::new("rustfmt")
            .args(&["src/calc.rs"])
            .status()
            .unwrap();
    };
    println!("cargo:rerun-if-changed=build.rs");
}
