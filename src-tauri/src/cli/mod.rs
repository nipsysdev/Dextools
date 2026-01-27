use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CliArgs {
    pub port: Option<u16>,
    pub data_dir: Option<String>,
}

/// Centralized CLI argument definitions
#[derive(Debug, Clone)]
pub struct CliArg {
    pub name: &'static str,
    pub short: &'static str,
    pub description: &'static str,
    pub default_value: Option<&'static str>,
}

pub const CLI_ARGS: &[CliArg] = &[
    CliArg {
        name: "port",
        short: "p",
        description: "Custom port for the storage node",
        default_value: Some("8089"),
    },
    CliArg {
        name: "data-dir",
        short: "d",
        description: "Custom data directory for storage",
        default_value: None,
    },
];

#[cfg(desktop)]
pub fn parse_cli_args(matches: &tauri_plugin_cli::Matches) -> CliArgs {
    let mut args = CliArgs::default();

    // Parse arguments dynamically from CLI_ARGS
    for arg in CLI_ARGS {
        if let Some(arg_value) = matches.args.get(arg.name) {
            if let Some(value_str) = arg_value.value.as_str() {
                match arg.name {
                    "port" => {
                        if let Ok(port) = value_str.parse::<u16>() {
                            args.port = Some(port);
                            println!("CLI argument: --port {}", port);
                        }
                    }
                    "data-dir" => {
                        args.data_dir = Some(value_str.to_string());
                        println!("CLI argument: --data-dir {}", value_str);
                    }
                    _ => {}
                }
            }
        }
    }

    args
}

#[cfg(not(desktop))]
pub fn parse_cli_args(_matches: &tauri_plugin_cli::Matches) -> CliArgs {
    CliArgs::default()
}

#[cfg(desktop)]
#[allow(clippy::unnecessary_get_then_check)]
pub fn handle_cli_help(matches: &tauri_plugin_cli::Matches) -> bool {
    if matches.args.get("help").is_some() {
        println!("Storeman - A simple Tauri desktop application for uploading and downloading files from Codex");
        println!();
        println!("Usage: storeman [OPTIONS]");
        println!();
        println!("Options:");

        // Generate help from centralized CLI_ARGS
        for arg in CLI_ARGS {
            let default = if let Some(default_val) = arg.default_value {
                format!(" (default: {})", default_val)
            } else {
                String::new()
            };
            println!(
                "  -{}, --{} <VALUE>      {}{}",
                arg.short, arg.name, arg.description, default
            );
        }

        println!("  -h, --help               Display this help message and exit");
        true
    } else {
        false
    }
}

#[cfg(not(desktop))]
pub fn handle_cli_help(_matches: &tauri_plugin_cli::Matches) -> bool {
    false
}
