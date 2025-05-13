use std::path::PathBuf;

fn build_asmlib(libdir: &PathBuf) {
    cc::Build::new()
        .file(libdir.join("wasmtime.c"))
        .file(libdir.join("wasmtime.S"))
        .flag("-fno-stack-protector")
        .compile("mylib");
}

fn main() {
    let libdir = std::env::current_dir().unwrap().join("native");
    build_asmlib(&libdir);
    println!("cargo::rustc-link-search=all={}", libdir.to_str().unwrap());
    println!("cargo::rustc-link-lib=static=mylib");
}
