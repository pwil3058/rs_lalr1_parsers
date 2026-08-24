// Copyright (c) 2026 Peter Williams <pwil3058@bigpond.net.au> <pwil3058@gmail.com>.
use std::{path::Path, process::Command};

fn main() {
    let alalr1_gen_path = "../target/debug/alalr1_gen";
    if Path::new(alalr1_gen_path).exists() {
        // println!("cargo:rerun-if-changed=src/parser.alaps");
        // println!("cargo::rerun-if-changed={alalr1_gen_path}");
        match Command::new(alalr1_gen_path)
            .args(["-f", "src/parser.alaps"])
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
        println!("cargo:rerun-if-changed=buildx");
    }
}
