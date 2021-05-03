// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=src/calc.alaps");
    println!("cargo:rerun-if-changed=src/calc.rs");
    Command::new("../../target/debug/alap_gen")
        .args(&["-f", "-e1", "src/calc.alaps"])
        .status()
        .unwrap();
    println!("cargo:rerun-if-changed=build.rs");
}
