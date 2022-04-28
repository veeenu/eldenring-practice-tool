use std::path::PathBuf;

fn main() {
    let exports_def = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("exports.def");

    println!(
        "cargo:rustc-cdylib-link-arg=/def:{}",
        exports_def.into_os_string().into_string().unwrap()
    );
}
