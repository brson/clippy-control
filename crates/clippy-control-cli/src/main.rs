#![allow(unused)]

use rx::prelude::*;
use rx::clap::{self, Parser as _};
use std::path::{Path, PathBuf};
use std::collections::HashMap;

fn main() -> AnyResult<()> {
    rx::extras::init();

    let cli = Cli::parse();
    cli.run()?;

    Ok(())
}

#[derive(clap::Parser)]
struct Cli {
    #[command(subcommand)]
    cmd: Command,
    #[command(flatten)]
    args: Args,
}

#[derive(clap::Subcommand)]
enum Command {
    Check(CheckCommand),
}

#[derive(clap::Args)]
struct Args {
    #[arg(default_value = "clippy-control.toml")]
    config_path: PathBuf,
}

#[derive(clap::Args)]
struct CheckCommand {
}

impl Cli {
    fn run(&self) -> AnyResult<()> {
        match &self.cmd {
            Command::Check(cmd) => cmd.run(&self.args),
        }
    }
}

impl CheckCommand {
    fn run(&self, args: &Args) -> AnyResult<()> {
        info!("hello world");

        let config = load_config(&args.config_path)?;
        run_clippy(&config)?;

        Ok(())
    }
}

struct Config {
    settings: HashMap<String, LintSetting>,
}

enum LintSetting {
    Warn, Allow, Deny, Forbid,
}

fn load_config(path: &Path) -> AnyResult<Config> {
    use rx::toml;
    use rx::toml::Value;
    use rx::toml::map::Map;
    use rx::serde::Deserialize;

    let buf = std::fs::read_to_string(path)
        .context(format!("unable to read config file {}", path.display()))?;
    let settings: Map<String, Value> = toml::from_str(&buf)?;

    todo!()
}

fn run_clippy(config: &Config) -> AnyResult<()> {
    todo!()
}
