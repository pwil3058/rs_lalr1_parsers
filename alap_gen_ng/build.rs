// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=src/alap_gen_ng.alaps");
    println!("cargo:rerun-if-changed=../target/debug/alap_gen");
    let status = Command::new("../target/debug/alap_gen")
        .args(&["-f", "-e0", "src/alap_gen_ng.alaps"])
        .status()
        .unwrap();
    if status.success() {
        Command::new("rustfmt")
            .args(&["src/alap_gen_ng.rs"])
            .status()
            .unwrap();
    };
    println!("cargo:rerun-if-changed=build.rs");
}
