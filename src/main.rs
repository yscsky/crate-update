use anyhow::Result;
use clap::Parser;
use crate_update::{filter_latest_crate, read_cargo, update_crate};
/// update crates
#[derive(Parser, Debug)]
struct CmdArgs {
    /// Cargo.toml file path
    #[arg(default_value_t=String::from("./Cargo.toml"))]
    file_path: String,
}

fn main() -> Result<()> {
    let cmd_args = CmdArgs::parse();
    let list = read_cargo(&cmd_args.file_path)?;
    let list = filter_latest_crate(list);
    println!(
        "update list: {}",
        list.iter()
            .map(|c| c.name.as_str())
            .collect::<Vec<&str>>()
            .join(",")
    );
    for c in list {
        if update_crate(&c)? {
            println!("{} update to {} success", c.name, c.version);
        } else {
            println!("{} update to {} failed", c.name, c.version);
        }
    }
    Ok(())
}
