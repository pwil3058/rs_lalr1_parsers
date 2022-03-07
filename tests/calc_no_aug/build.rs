// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=src/calc_no_aug.laps");
    println!("cargo:rerun-if-changed=../../target/debug/lap_gen");
    match Command::new("../../target/debug/lap_gen")
        .args(&["-f", "src/calc_no_aug.laps"])
        .status()
    {
        Ok(status) => {
            if status.success() {
                Command::new("rustfmt")
                    .args(&["src/calc_no_aug.rs"])
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
