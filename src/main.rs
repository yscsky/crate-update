use anyhow::Result;
use clap::Parser;
use crate_update::read_cargo;
/// update crates
#[derive(Parser, Debug)]
struct CmdArgs {
    /// Cargo.toml file path
    #[arg(default_value_t=String::from("./Cargo.toml"))]
    file_path: String,
}

fn main() -> Result<()> {
    let cmd_args = CmdArgs::parse();
    // 读取Cargo配置
    read_cargo(&cmd_args.file_path)?;
    Ok(())
}
