use anyhow::{anyhow, Result};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::process::Command;
use std::{fs, sync::mpsc, thread};
use toml::{Table, Value};

const USER_AGENT:&str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/116.0.0.0 Safari/537.36";

#[derive(Debug)]
pub struct Crate {
    pub name: String,
    pub version: String,
    pub features: String,
}

impl Crate {
    pub fn new_by_name(name: String) -> Self {
        Self {
            name,
            version: String::new(),
            features: String::new(),
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
                        let mut list = Vec::new();
                        for v in features {
                            if let Value::String(f) = v {
                                list.push(f.clone());
                            }
                        }
                        c.features = list.join(",");
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

pub fn filter_latest_crate(list: Vec<Crate>) -> Vec<Crate> {
    let (tx, rx) = mpsc::sync_channel(10);
    for c in list {
        let tx = tx.clone();
        thread::spawn(move || {
            let ver = c.version.clone();
            match query_crate_latest_version(c) {
                Ok(latest) => {
                    if latest.version != ver {
                        tx.send(latest).expect("send crate error");
                    }
                }
                Err(e) => println!("query crate lastest version error: {e:?}"),
            }
        });
    }
    drop(tx);
    let mut res = Vec::new();
    for c in rx {
        res.push(c);
    }
    res
}

#[test]
fn test_filter_latest_crate() {
    let list = vec![
        Crate::new_by_name("anyhow".into()),
        Crate::new_by_name("tokio".into()),
        Crate {
            name: "serde".into(),
            version: "1.0.188".into(),
            features: "".into(),
        },
    ];
    let list = filter_latest_crate(list);
    println!("list: {:#?}", list);
}

#[derive(Deserialize, Debug)]
struct LatestResp {
    #[serde(rename = "crate")]
    crate_obj: CrateObj,
}

#[derive(Deserialize, Debug)]
struct CrateObj {
    max_stable_version: String,
}

pub fn query_crate_latest_version(c: Crate) -> Result<Crate> {
    let resp: LatestResp = Client::builder()
        .user_agent(USER_AGENT)
        .build()?
        .get(format!("https://crates.io/api/v1/crates/{}", c.name))
        .send()?
        .json()?;
    Ok(Crate {
        name: c.name,
        version: resp.crate_obj.max_stable_version,
        features: c.features,
    })
}

#[test]
fn test_query_crate_latest_version() {
    match query_crate_latest_version(Crate::new_by_name("tokio".into())) {
        Ok(c) => println!("crate: {:?}", c),
        Err(e) => println!("err: {e:?}"),
    }
}

pub fn update_crate(c: &Crate) -> Result<bool> {
    let mut cargo = Command::new("cargo");
    cargo.arg("add").arg(format!("{}@{}", c.name, c.version));
    if !c.features.is_empty() {
        cargo.arg("--features").arg(&c.features);
    }
    let status = cargo.status()?;
    Ok(status.success())
}

#[test]
fn test_update_crate() {
    match update_crate(&Crate {
        name: "tokio".into(),
        version: "1.32.0".into(),
        features: "rt-multi-thread,macros,sync,time".into(),
    }) {
        Ok(success) => println!("success: {success}"),
        Err(e) => println!("err: {e:?}"),
    }
}
