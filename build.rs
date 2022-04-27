fn main() {
    lalrpop::process_root().unwrap();
    cxx_build::bridge("src/eval/operation.rs")
        .file("cpp/nix.cc")
        .flag_if_supported("-std=c++17")
        .flag_if_supported("-U_FORTIFY_SOURCE") // Otherwise builds with `-O0` raise a lot of warnings
        .compile("nickel-lang");
    println!("cargo:rustc-link-lib=nixstore");
}
