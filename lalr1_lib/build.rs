// Copyright (c) 2026 Peter Williams <pwil3058@bigpond.net.au> <pwil3058@gmail.com>.
use std::{path::Path, process::Command};

fn main() {
    let lap_gen_path = "../target/debug/lap_gen";
    if Path::new(lap_gen_path).exists() {
        println!("cargo:rerun-if-changed=src/lap_gen.laps");
        println!("cargo:rerun-if-changed=../target/debug/lap_gen");
        println!("cargo::rerun-if-changed={lap_gen_path}");
        match Command::new(lap_gen_path)
            .args(["-f", "src/lap_gen.laps"])
            .status()
        {
            Ok(status) => {
                if status.success() {
                    Command::new("rustfmt")
                        .args(["src/lap_gen.rs"])
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
