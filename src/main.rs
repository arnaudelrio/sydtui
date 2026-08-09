#![doc = include_str!("../README.md")]

use std::io;

use clap::Parser;

/// The main module for the sydtui application.
pub mod app;
/// The configuration module for the sydtui application.
pub mod config;
/// The events module for the sydtui application.
pub mod events;
/// The key bindings module for the sydtui application.
pub mod key_bindings;
/// The service module for the sydtui application.
pub mod service;
/// The service groups module for the sydtui application.
pub mod service_groups;
/// The UI module for the sydtui application.
pub mod ui;

use crate::app::App;

/// The main entry point for the sydtui application.
pub fn main() -> io::Result<()> {
    let args = Cli::parse();
    let mut app = App::init()?;

    if args.cli() {
        args.run_cli(&mut app)
    } else {
        ratatui::run(|terminal| app.run(terminal))
    }
}

/// The command-line interface for the sydtui application.
/// 
/// Usage:
/// ```
/// Usage: sydtui [OPTIONS]
///
/// Options:
///   -a, --activate <SERVICE_GROUP>
///          Toggle the activation of a group of services
///
///   -e, --enable <SERVICE_GROUP>
///          Toggle the enablement of a group of services
///
///   -l, --list
///          List all available service groups
///
///   -h, --help
///          Print help (see a summary with '-h')
///
///   -V, --version
///          Print version
/// ```
#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    /// Toggle the activation of a group of services
    #[arg(short = 'a', long = "activate", value_name = "SERVICE_GROUP")]
    activate: Option<String>,

    /// Toggle the enablement of a group of services
    #[arg(short = 'e', long = "enable", value_name = "SERVICE_GROUP")]
    enable: Option<String>,

    /// List all available service groups
    #[arg(short = 'l', long = "list")]
    list: bool,
}

/// Parses the command-line arguments and runs the appropriate action.
impl Cli {
    /// Returns `true` if the CLI should run, `false` otherwise.
    fn cli(&self) -> bool {
        self.activate.is_some() || self.enable.is_some() || self.list
    }

    /// Runs the CLI with the given application state.
    fn run_cli(&self, app: &mut App) -> io::Result<()> {
        if self.list {
            app.list_service_groups();
        }

        if let Some(service) = &self.activate {
            let active = app.toggle_activate(Some(service.clone()));
            if let Ok(active) = active {
                let active = if active { "active" } else { "inactive" };
                println!("Services {} is {}", service, active)
            } else {
                println!("Failed to toggle activate for service: {}", service)
            }
        }

        if let Some(service) = &self.enable {
            let enabled = app.toggle_enabled(Some(service.clone()));
            if let Ok(enabled) = enabled {
                let enabled = if enabled { "enabled" } else { "disabled" };
                println!("Services {} is {}", service, enabled)
            } else {
                println!("Failed to toggle enable for service: {}", service)
            }
        }

        app.save()
    }
}
