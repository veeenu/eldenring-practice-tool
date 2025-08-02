fn main() {
    embed_resource::compile("./src/resources.rc", embed_resource::NONE)
        .manifest_required()
        .expect("Couldn't embed resource");
}
