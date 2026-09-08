use std::collections::{BTreeMap, HashSet};
use std::fmt::Write;
use std::fs;
use std::path::Path;

use anyhow::{bail, ensure, Context};
use once_cell::sync::Lazy;
use practice_tool_tasks::params::{checkout_paramdex, codegen_param_names};
use quick_xml::events::Event;
use quick_xml::{Reader, XmlVersion};
use regex::Regex;

use crate::{project_root, Result};

pub(crate) fn codegen() -> Result<()> {
    checkout_paramdex()?;

    let source = generate_param_data(&project_root().join("target/Paramdex"))?;

    fs::write(project_root().join("lib/libeldenring/src/codegen/param_data.rs"), source)?;

    codegen_param_names(
        "target/Paramdex/ER/Names",
        "lib/libeldenring/src/codegen/param_names.json",
    )?;

    Ok(())
}

fn snake_case(name: &str) -> String {
    static CAPITALS: Lazy<Regex> = Lazy::new(|| Regex::new(r"[A-Z]+").unwrap());
    static UNDERSCORES: Lazy<Regex> = Lazy::new(|| Regex::new(r"_+").unwrap());

    let mut result = String::new();

    if let Some(first) = name.chars().next() {
        result.push(first);
        result.push_str(&CAPITALS.replace_all(&name[first.len_utf8()..], "_$0"));
    }

    UNDERSCORES.replace_all(&result.to_lowercase(), "_").into_owned()
}

fn fix_name(name: &str) -> String {
    if name.starts_with(|c: char| c.is_ascii_digit()) {
        format!("field{name}")
    } else if name == "type" {
        "ty".into()
    } else {
        name.into()
    }
}

fn slug(name: &str) -> String {
    name.chars().filter(char::is_ascii_alphabetic).collect::<String>().to_ascii_lowercase()
}

struct Field {
    name: String,
    ty: String,
    bitfield_size: Option<usize>,
    flags: Vec<Field>,
}

impl Field {
    fn parse(definition: &str) -> Result<Self> {
        static DEFINITION: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"^(\w+)\s+(\w+)(?:\[(\d+)\]|:(\d+))?").unwrap());

        let captures = DEFINITION
            .captures(definition)
            .with_context(|| format!("Couldn't parse field: {definition}"))?;

        let ty = match &captures[1] {
            "s8" => "i8",
            "s16" => "i16",
            "s32" => "i32",
            "u8" | "dummy8" | "fixstr" => "u8",
            "u16" | "fixstrW" => "u16",
            "u32" => "u32",
            "f32" => "f32",
            other => bail!("Unsupported field type {other} in {definition}"),
        };

        let bitfield_size = if captures.get(4).is_some() {
            Some(match ty {
                "u8" => 8,
                "u16" => 16,
                "u32" => 32,
                _ => bail!("Unsupported bitfield type in {definition}"),
            })
        } else {
            None
        };

        let ty = match captures.get(3) {
            Some(count) => format!("[{ty}; {}]", count.as_str().parse::<usize>()?),
            None => ty.into(),
        };

        Ok(Self { name: captures[2].into(), ty, bitfield_size, flags: Vec::new() })
    }
}

fn dedup_fields(fields: &mut [Field]) {
    let mut names = HashSet::new();
    let mut index = 0;

    for field in fields {
        let name = snake_case(&field.name);

        if names.contains(&name) {
            field.name = format!("{}_{index}", field.name);
            index += 1;
        }

        names.insert(name);
    }
}

fn parse_layout(xml: &str) -> Result<Vec<Field>> {
    let mut reader = Reader::from_str(xml);
    let mut fields = Vec::new();
    let mut bits = Vec::new();
    let mut bitfield_index = 0;
    let mut depth = 0;
    let mut fields_depth = None;

    loop {
        let event = reader.read_event()?;
        let empty = matches!(event, Event::Empty(_));

        match event {
            Event::Start(element) | Event::Empty(element) => {
                if depth == 1 && element.name().as_ref() == b"Fields" && !empty {
                    fields_depth = Some(depth + 1);
                } else if fields_depth == Some(depth) && element.name().as_ref() == b"Field" {
                    let definition = element
                        .try_get_attribute("Def")?
                        .context("Field is missing its Def attribute")?;

                    let definition = definition
                        .decoded_and_normalized_value(XmlVersion::Implicit1_0, reader.decoder())?;

                    let field = Field::parse(&definition)?;

                    if let Some(size) = field.bitfield_size {
                        let ty = field.ty.clone();
                        bits.push(field);
                        if bits.len() == size {
                            dedup_fields(&mut bits);
                            fields.push(Field {
                                name: format!("bitfield{bitfield_index}"),
                                ty,
                                bitfield_size: None,
                                flags: std::mem::take(&mut bits),
                            });
                            bitfield_index += 1;
                        }
                    } else {
                        fields.push(field);
                    }
                }

                if !empty {
                    depth += 1;
                }
            },
            Event::End(_) => {
                if fields_depth == Some(depth) {
                    fields_depth = None;
                }

                depth -= 1;
            },
            Event::Eof => break,
            _ => {},
        }
    }

    ensure!(depth == 0, "Unexpected end of XML document");

    dedup_fields(&mut fields);

    Ok(fields)
}

fn generate_param_data(paramdex: &Path) -> Result<String> {
    let definitions = paramdex.join("ER/Defs");
    let mut layouts = BTreeMap::new();

    for entry in
        fs::read_dir(&definitions).with_context(|| format!("Reading {}", definitions.display()))?
    {
        let path = entry?.path();

        if path.extension().and_then(|s| s.to_str()) != Some("xml") {
            continue;
        }

        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .context("Invalid XML filename")?
            .replace("_ST", "");

        let key = slug(&name);

        if key == "defaultkeyassign" {
            continue;
        }

        let fields = fs::read_to_string(&path)
            .map_err(anyhow::Error::from)
            .and_then(|xml| parse_layout(&xml))
            .with_context(|| format!("Parsing {}", path.display()))?;

        layouts.insert(key, (name, fields));
    }

    ensure!(!layouts.is_empty(), "No parameter definitions in {}", definitions.display());

    let mut source = String::from(
        r#"// **********************************
// *** AUTOGENERATED, DO NOT EDIT ***
// **********************************
use std::collections::HashMap;
use std::ffi::c_void;

use once_cell::sync::Lazy;
use macro_param::ParamStruct;
use crate::prelude::*;

unsafe fn get_lambda<T: ParamStruct>() -> BoxedVisitorLambda {
    Box::new(|ptr, v| {
        if let Some(r) = (ptr as *mut T).as_mut() {
            r.visit(&mut *v);
        }
    })
}

type BoxedVisitorLambda = Box<dyn Fn(*const c_void, &mut dyn ParamVisitor) + Send + Sync>;

pub static PARAM_VTABLE: Lazy<HashMap<String, BoxedVisitorLambda>> = Lazy::new(|| {
    [
"#,
    );

    for (name, _) in layouts.values() {
        writeln!(source, "        (\"{name}\".to_string(), unsafe {{ get_lambda::<{name}>() }}),")?;
    }

    source.push_str("    ].into_iter().collect()\n});");

    for (name, fields) in layouts.values() {
        writeln!(source, "\n#[derive(ParamStruct, Debug)]\n#[repr(C)]\npub struct {name} {{")?;

        for field in fields {
            for (index, flag) in field.flags.iter().enumerate() {
                writeln!(source, "    #[bitflag({}, {index})]", fix_name(&flag.name))?;
            }

            writeln!(source, "    pub {}: {},", fix_name(&snake_case(&field.name)), field.ty)?;
        }

        source.push_str("}\n");
    }
    Ok(source)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_xml_fields_and_preserves_names() -> Result<()> {
        let fields = parse_layout(
            r#"<?xml version="1.0"?>
            <PARAMDEF><Fields>
                <Field Def="s32 type = -1"><Description>ignored</Description></Field>
                <Field Def="fixstrW title[16]" />
                <Field Def="f32 2Speed = 1" />
                <Field Def="u8 fooBar" />
                <Field Def="u8 foo_bar" />
            </Fields></PARAMDEF>"#,
        )?;

        assert_eq!(fields.len(), 5);
        assert_eq!(fix_name(&fields[0].name), "ty");
        assert_eq!(fields[0].ty, "i32");
        assert_eq!(fields[1].ty, "[u16; 16]");
        assert_eq!(fix_name(&snake_case(&fields[2].name)), "field2_speed");
        assert_eq!(fields[4].name, "foo_bar_0");
        assert_eq!(snake_case("AIAttackParam"), "a_iattack_param");

        Ok(())
    }

    #[test]
    fn groups_bitflags_in_order() -> Result<()> {
        let mut xml = String::from("<PARAMDEF><Fields>");

        for index in 0..8 {
            write!(xml, "<Field Def=\"u8 flag{index}:1\" />")?;
        }

        xml.push_str("<Field Def=\"s32 value\" /></Fields></PARAMDEF>");

        let fields = parse_layout(&xml)?;

        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].name, "bitfield0");
        assert_eq!(fields[0].ty, "u8");
        assert_eq!(fields[0].flags.len(), 8);
        assert_eq!(fields[0].flags[7].name, "flag7");
        assert_eq!(fields[1].name, "value");

        Ok(())
    }

    #[test]
    fn reports_invalid_definitions() {
        assert!(Field::parse("invalid").is_err());
        assert!(Field::parse("unknown value").is_err());
        assert!(parse_layout("<PARAMDEF><Fields><Field /></Fields></PARAMDEF>").is_err());
        assert!(parse_layout("<PARAMDEF><Fields></PARAMDEF>").is_err());
    }

    #[test]
    #[ignore = "requires a local target/Paramdex checkout"]
    fn generates_local_paramdex() -> Result<()> {
        let source = generate_param_data(&project_root().join("target/Paramdex"))?;

        assert!(source.contains("pub struct EquipParamWeapon"));
        assert!(!source.contains("pub struct DefaultKeyAssign"));

        Ok(())
    }
}
