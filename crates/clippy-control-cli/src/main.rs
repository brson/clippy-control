#![allow(unused)]

use rx::prelude::*;
use rx::clap::{self, Parser as _};
use rx::serde;
use std::path::{Path, PathBuf};
use std::collections::BTreeMap;

fn main() -> AnyResult<()> {
    rx::extras::init();

    let cli = Cli::parse();
    cli.run()?;

    Ok(())
}

#[derive(clap::Parser)]
struct Cli {
    #[command(subcommand)]
    cmd: Option<Command>,
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

#[derive(Default)]
#[derive(clap::Args)]
struct CheckCommand {
}

impl Cli {
    fn run(self) -> AnyResult<()> {
        match self.cmd.unwrap_or_else(|| {
            Command::Check(CheckCommand::default())
        }) {
            Command::Check(cmd) => cmd.run(&self.args),
        }
    }
}

impl CheckCommand {
    fn run(&self, args: &Args) -> AnyResult<()> {
        let config = load_config(&args.config_path)?;
        run_clippy(&config)?;

        Ok(())
    }
}

struct Config {
    settings: BTreeMap<String, LintSetting>,
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
    let settings: AnyResult<BTreeMap<String, LintSetting>> = settings.into_iter()
        .map(|(lint_name, setting)| {
            let setting = LintSetting::from_toml(&setting)
                .context(format!("parsing lint '{lint_name}'"))?;
            Ok((lint_name, setting))
        })
        .collect();
    let settings = settings?;

    Ok(Config { settings })
}

fn run_clippy(config: &Config) -> AnyResult<()> {
    use std::process::Command;

    let settings_args: Vec<String> = config.settings.iter()
        .map(|(lint_name, setting)| {
            setting.clippy_arg(lint_name)
        }).collect();

    let status = Command::new("cargo")
        .arg("clippy")
        .arg("--")
        .args(&settings_args)
        .status()?;

    match status.code() {
        Some(code) => {
            std::process::exit(code);
        }
        None => {
            bail!("cargo-clippy terminated without exit code");
        }
    }
}

impl LintSetting {
    fn from_toml(value: &rx::toml::Value) -> AnyResult<LintSetting> {
        use rx::toml::Value;

        Ok(match value {
            Value::String(s) => {
                match s.as_str() {
                    "warn" => LintSetting::Warn,
                    "allow" => LintSetting::Allow,
                    "deny" => LintSetting::Deny,
                    "forbid" => LintSetting::Forbid,
                    _ => bail!("unrecognized value '{s}'"),
                }
            }
            _ => {
                bail!("value '{}' not a string", value);
            }
        })
    }

    fn clippy_arg(&self, lint_name: &str) -> String {
        match self {
            LintSetting::Warn => {
                format!("-Wclippy::{lint_name}")
            }
            LintSetting::Allow => {
                format!("-Aclippy::{lint_name}")
            }
            LintSetting::Forbid => {
                format!("-Fclippy::{lint_name}")
            }
            LintSetting::Deny => {
                format!("-Dclippy::{lint_name}")
            }
        }
    }
}

