use std::fs;

use practice_tool_tasks::params::{checkout_paramdex, codegen_param_data, codegen_param_names};

use crate::{project_root, Result};

pub(crate) fn codegen() -> Result<()> {
    checkout_paramdex()?;

    let source = codegen_param_data(&project_root().join("target/Paramdex"), "ER")?;

    fs::write(project_root().join("lib/libeldenring/src/codegen/param_data.rs"), source)?;

    codegen_param_names(
        "target/Paramdex/ER/Names",
        "lib/libeldenring/src/codegen/param_names.json",
    )?;

    Ok(())
}
