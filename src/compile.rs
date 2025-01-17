use crate::utils::*;
use std::process::Command;

fn run_and_printerror(command: &mut Command) {
    println!("Running: `{:?}`", command);
    match command.status() {
        Ok(status) => {
            if !status.success() {
                panic!("Failed: `{:?}` ({})", command, status);
            }
        }
        Err(error) => {
            panic!("Failed: `{:?}` ({})", command, error);
        }
    }
}

#[allow(unused)]
fn print_llvm_version() {
    let mut cmake = Command::new("cmake");
    cmake.args(&["-version"]);
    run_and_printerror(&mut cmake);
}

pub fn build(to_build: &str) -> Result<(), String> {
    let _repo = match to_build {
        "enzyme" => build_enzyme(),
        "rustc" => build_rustc(),
        _ => return Err("Unknown argument. Try enzyme or rustc.".to_owned()),
    };
    Ok(())
}

fn build_enzyme() {
    let llvm_dir = get_llvm_build_path()
        .join("lib")
        .join("cmake")
        .join("llvm");
    let llvm_dir = "-DLLVM_DIR=".to_owned() + llvm_dir.to_str().unwrap();
    let llvm_external_lit = get_rustc_repo_path()
        .join("src")
        .join("llvm-project")
        .join("llvm")
        .join("utils")
        .join("lit")
        .join("lit.py");
    let llvm_external_lit = "-DLLVM_EXTERNAL_LIT=".to_owned() + llvm_external_lit.to_str().unwrap();
    let llvm_external_lib = "-DENZYME_EXTERNAL_SHARED_LIB=ON".to_owned();
    let build_type = "-DCMAKE_BUILD_TYPE=Release";
    let mut cmake = Command::new("cmake");
    let mut ninja = Command::new("ninja");
    let mut ninja_check = Command::new("ninja");
    let build_path = get_enzyme_build_path();
    if !std::path::Path::new(&build_path).exists() {
        std::fs::create_dir(&build_path).unwrap();
    }
    cmake
        .args(&[
            "-G",
            "Ninja",
            "..",
            build_type,
            &llvm_external_lib,
            &llvm_dir,
            &llvm_external_lit,
        ])
        .current_dir(&build_path.to_str().unwrap());
    ninja.current_dir(&build_path.to_str().unwrap());
    ninja_check
        .args(&["check-enzyme"])
        .current_dir(&build_path.to_str().unwrap());
    run_and_printerror(&mut cmake);
    run_and_printerror(&mut ninja);
    run_and_printerror(&mut ninja_check);
}

fn build_rustc() {
    let mut cargo = Command::new("cargo");
    let mut configure = Command::new("./configure");
    let mut x = Command::new("x");
    let mut rustup = Command::new("rustup");

    let build_path = get_rustc_build_path();
    if !std::path::Path::new(&build_path).exists() {
        std::fs::create_dir(&build_path).unwrap();
    }
    let x_path = std::path::Path::new("src").join("tools").join("x");
    let toolchain_path = get_rustc_stage2_path();

    cargo
        .args(&["install", "--path", x_path.to_str().unwrap()])
        .current_dir(&build_path.to_str().unwrap());

    configure
        .args(&[
            "--enable-llvm-link-shared",
            "--enable-llvm-plugins",
            "--release-channel=nightly",
            "--enable-llvm-assertions",
            "--enable-clang",
            "--enable-lld",
            "--enable-option-checking",
            "--enable-ninja",
        ])
        .current_dir(&build_path.to_str().unwrap());

    x.args(&["build", "--stage", "2"])
        .current_dir(&build_path.to_str().unwrap());

    rustup
        .args(&[
            "toolchain",
            "link",
            "enzyme",
            toolchain_path.to_str().unwrap(),
        ])
        .current_dir(&build_path.to_str().unwrap());

    run_and_printerror(&mut cargo);
    run_and_printerror(&mut configure);
    run_and_printerror(&mut x);
    run_and_printerror(&mut rustup);
}
