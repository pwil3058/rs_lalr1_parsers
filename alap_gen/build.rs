// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=src/alap_gen.alaps");
    println!("cargo:rerun-if-changed=../target/debug/alap_gen");
    if let Ok(status) = Command::new("../target/debug/alap_gen")
        .args(&["-f", "-e1", "src/alap_gen.alaps"])
        .status()
    {
        if status.success() {
            Command::new("rustfmt")
                .args(&["src/alap_gen.rs"])
                .status()
                .unwrap();
        };
    }
    println!("cargo:rerun-if-changed=build.rs");
}
