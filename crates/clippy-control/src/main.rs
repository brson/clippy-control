#![allow(unused)]

use rmx::prelude::*;
use rmx::clap::{self, Parser as _};
use rmx::serde;
use std::path::{Path, PathBuf};
use std::collections::BTreeMap;

fn main() -> AnyResult<()> {
    rmx::extras::init();

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
    #[arg(long)]
    fix: bool,
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
        run_clippy(&config, args)?;

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
    use rmx::toml;
    use rmx::toml::Value;
    use rmx::toml::map::Map;
    use rmx::serde::Deserialize;

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

fn run_clippy(
    config: &Config,
    args: &Args,
) -> AnyResult<()> {
    use std::process::Command;

    let settings_args: Vec<String> = config.settings.iter()
        .map(|(lint_name, setting)| {
            setting.clippy_arg(lint_name)
        }).collect();

    let mut cmd = Command::new("cargo");
    cmd.arg("clippy");
    if args.fix {
        cmd.arg("--fix");
    }
    cmd.arg("--").args(&settings_args);
    let status = cmd.status()?;

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
    fn from_toml(value: &rmx::toml::Value) -> AnyResult<LintSetting> {
        use rmx::toml::Value;

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

