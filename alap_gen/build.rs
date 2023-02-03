// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
#[cfg(not(feature = "bootstrap"))]
use std::{process::Command, path::Path};

#[cfg(not(feature = "bootstrap"))]
fn main() {
    let alap_gen_path = "../target/debug/alap_gen";
    if Path::new(alap_gen_path).exists() {
        println!("cargo:rerun-if-changed=src/alap_gen.alaps");
        println!("cargo:rerun-if-changed=../target/debug/alap_gen");
        println!("cargo::rerun-if-changed={alap_gen_path}");
        match Command::new(alap_gen_path)
            .args(["-f", "src/alap_gen.alaps"])
            .status()
        {
            Ok(status) => {
                if status.success() {
                    Command::new("rustfmt")
                        .args(["src/alap_gen.rs"])
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

#[cfg(feature = "bootstrap")]
fn main() {}
