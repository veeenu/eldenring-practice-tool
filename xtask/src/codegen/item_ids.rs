use crate::{project_root, Result};

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ItemIDNode {
    Leaf { node: String, value: u32 },
    Node { node: String, children: Vec<ItemIDNode> },
}

fn get_item_ids_yml() -> Result<serde_yaml::Value> {
    let file = std::fs::File::open(
        project_root()
            .join("xtask")
            .join("src")
            .join("codegen")
            .join("item_ids.yml"),
    )?;
    serde_yaml::from_reader(file).map_err(|e| e.into())
}

pub(crate) fn codegen() -> Result<()> {
    
}
