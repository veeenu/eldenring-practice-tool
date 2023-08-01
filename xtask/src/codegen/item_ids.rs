use std::convert::TryFrom;
use std::path::PathBuf;

use serde::Serialize;
use serde_yaml::Value;

use crate::{project_root, DynError, Result};

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum ItemIDNode {
    Leaf { node: String, value: u32 },
    Node { node: String, children: Vec<ItemIDNode> },
}

impl TryFrom<(Value, Value)> for ItemIDNode {
    type Error = DynError;

    fn try_from((k, v): (Value, Value)) -> Result<Self> {
        match (k, v) {
            (Value::String(s), Value::Number(n)) => {
                Ok(ItemIDNode::Leaf { node: s, value: n.as_u64().unwrap() as u32 })
            },
            (Value::String(s), Value::Mapping(m)) => Ok(ItemIDNode::Node {
                node: s,
                children: m.into_iter().map(|(k, v)| ItemIDNode::try_from((k, v))).try_fold(
                    Vec::new(),
                    |mut o: Vec<_>, i: Result<ItemIDNode>| {
                        let i = i?;
                        o.push(i);
                        Result::Ok(o)
                    },
                )?,
            }),
            (a, b) => Err(format!("invalid value {:?} {:?}", a, b).into()),
        }
    }
}

fn item_ids_yml_path() -> PathBuf {
    project_root().join("xtask").join("src").join("codegen").join("item_ids.yml")
}

fn item_ids_json_path() -> PathBuf {
    project_root().join("practice-tool").join("src").join("widgets").join("item_ids.json")
}

fn get_item_ids_yml() -> Result<serde_yaml::Value> {
    let file = std::fs::File::open(item_ids_yml_path())?;
    serde_yaml::from_reader(file).map_err(|e| e.into())
}

pub(crate) fn codegen() -> Result<()> {
    let val = get_item_ids_yml()?;

    let v: Result<Vec<ItemIDNode>> = match val {
        Value::Mapping(m) => m.into_iter().map(|(k, v)| ItemIDNode::try_from((k, v))).collect(),
        _ => Err("invalid input format".into()),
    };

    let v = v?;

    serde_json::to_writer(std::fs::File::create(item_ids_json_path())?, &v)?;

    Ok(())
}
