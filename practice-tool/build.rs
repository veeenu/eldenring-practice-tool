fn main() {
    embed_resource::compile("./src/practice-tool.rc", embed_resource::NONE);
    // println!("cargo:rustc-cdylib-link-arg=/DEF:lib/no-logo/exports.def");
}
