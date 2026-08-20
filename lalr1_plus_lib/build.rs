// Copyright (c) 2026 Peter Williams <pwil3058@bigpond.net.au> <pwil3058@gmail.com>.
#[cfg(not(feature = "bootstrap"))]
use std::{path::Path, process::Command};

#[cfg(not(feature = "bootstrap"))]
fn main() {
    let lalr1_plus_gen_path = "../target/debug/lalr1_plus_gen";
    if Path::new(lalr1_plus_gen_path).exists() {
        println!("cargo:rerun-if-changed=src/alap_gen.alaps");
        println!("cargo::rerun-if-changed={lalr1_plus_gen_path}");
        match Command::new(lalr1_plus_gen_path)
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
