use anyhow::{anyhow, Result};
use reqwest::blocking::Client;
use serde::Deserialize;
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

pub fn filter_lastest_crate(list: Vec<Crate>) -> Vec<Crate> {
    let (tx, rx) = mpsc::sync_channel(10);
    for c in list {
        let tx = tx.clone();
        thread::spawn(move || {
            let ver = c.version.clone();
            match query_crate_lastest_version(c) {
                Ok(lastest) => {
                    if lastest.version != ver {
                        tx.send(lastest).expect("send crate error");
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
fn test_filter_lastest_crate() {
    let list = vec![
        Crate::new_by_name("anyhow".into()),
        Crate::new_by_name("tokio".into()),
        Crate {
            name: "serde".into(),
            version: "1.0.188".into(),
            features: "".into(),
        },
    ];
    let list = filter_lastest_crate(list);
    println!("list: {:#?}", list);
}

#[derive(Deserialize, Debug)]
struct LastestResp {
    #[serde(rename = "crate")]
    crate_obj: CrateObj,
}

#[derive(Deserialize, Debug)]
struct CrateObj {
    max_stable_version: String,
}

pub fn query_crate_lastest_version(c: Crate) -> Result<Crate> {
    let resp: LastestResp = Client::builder()
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
fn test_query_crate_lastest_version() {
    match query_crate_lastest_version(Crate::new_by_name("tokio".into())) {
        Ok(c) => println!("crate: {:?}", c),
        Err(e) => println!("err: {e:?}"),
    }
}
