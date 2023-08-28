use anyhow::{anyhow, Result};
use std::fs;
use toml::{Table, Value};

#[derive(Debug)]
pub struct Crate {
    pub name: String,
    pub version: String,
    pub features: Vec<String>,
}

impl Crate {
    pub fn new_by_name(name: String) -> Self {
        Self {
            name,
            version: String::new(),
            features: Vec::new(),
        }
    }
}

pub fn read_cargo(path: &str) -> Result<Vec<Crate>> {
    let mut crates = Vec::new();
    let data = fs::read_to_string(path)?;
    let table = &data.parse::<Table>()?;
    if let Some(Value::Table(table)) = table.get("dependencies") {
        for (name, value) in table {
            let mut c = Crate::new_by_name(name.clone());
            match value {
                Value::String(version) => c.version = version.clone(),
                Value::Table(table) => {
                    if let Some(Value::String(version)) = table.get("version") {
                        c.version = version.clone();
                    }
                    if let Some(Value::Array(features)) = table.get("features") {
                        for v in features {
                            if let Value::String(f) = v {
                                c.features.push(f.clone());
                            }
                        }
                    }
                }
                _ => {}
            }
            crates.push(c);
        }
    } else {
        return Err(anyhow!("Cargo no dependencies"));
    }
    Ok(crates)
}

#[test]
fn test_read_cargo() {
    match read_cargo("Cargo.toml") {
        Ok(list) => {
            println!("list: {:#?}", list);
        }
        Err(e) => println!("err: {e:?}"),
    }
}


