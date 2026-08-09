use std::{
    fs::read_to_string,
    io::{self},
    path::PathBuf,
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};

use crate::{App, key_bindings::Keybindings, service::Service, service_groups::ServiceGroup};

/// The main configuration struct.
#[derive(Default, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct Config {
    /// The keybindings for the application.
    pub keybindings: Keybindings,
    /// The groups of services in the configuration.
    pub groups: Vec<ConfigGroup>,
}

/// A group of services in the configuration.
#[derive(Default, Deserialize, Serialize, Clone)]
pub struct ConfigGroup {
    /// The name of the group.
    name: String,
    /// Whether the group is active.
    is_active: bool,
    /// Whether the group is enabled.
    is_enabled: bool,
    /// The services in the group.
    services: Vec<ConfigService>,
}

/// A service in the configuration.
#[derive(Default, Deserialize, Serialize, Clone)]
pub struct ConfigService {
    /// The name of the service.
    name: String,
    /// The description of the service.
    description: String,
    /// The path to the service.
    path: PathBuf,
    /// Whether the service is active.
    is_active: bool,
    /// Whether the service is enabled.
    is_enabled: bool,
}

/// The main configuration implementation.
impl Config {
    /// Returns the path to the configuration file.
    /// 
    /// Evaluates the `SYDTUI_CONFIG` environment variable, falling back to `$HOME/.config/sydtui/config.toml` if not set.
    /// 
    /// # Returns
    /// 
    /// The path to the configuration file.
    pub fn get_config_path() -> PathBuf {
        let path = PathBuf::from(std::env::var("SYDTUI_CONFIG").unwrap_or_else(|_| {
            format!(
                "{}/.config/sydtui/config.toml",
                std::env::var("HOME").unwrap()
            )
        }));
        path
    }

    /// Returns the contents of the configuration file.
    /// 
    /// # Returns
    /// 
    /// The contents of the configuration file as a string.
    pub fn get_config_file_contents() -> io::Result<String> {
        let config_path = Self::get_config_path();
        let contents = read_to_string(config_path.clone()).unwrap_or_else(|_| {
            std::fs::create_dir_all(&config_path.parent().unwrap()).unwrap();
            std::fs::File::create(&config_path).unwrap();
            String::new()
        });
        Ok(contents)
    }

    /// Parses the configuration file and loads it into the application.
    /// 
    /// # Arguments
    /// 
    /// * `app` - The application instance to load the configuration into.
    pub fn load_config(app: &mut App) -> io::Result<()> {
        let config_file = Self::get_config_file_contents()?;
        let config: Config = toml::from_str(&config_file).unwrap_or_default();
        let service_groups: Vec<ServiceGroup> = config
            .groups
            .iter()
            .map(|group| ServiceGroup {
                name: group.name.clone(),
                is_active: group.is_active.clone(),
                is_enabled: group.is_enabled.clone(),
                services: group
                    .services
                    .iter()
                    .map(|service| {
                        Arc::new(Mutex::new(Service {
                            name: service.name.clone(),
                            description: service.description.clone(),
                            path: service.path.clone(),
                            is_active: service.is_active.clone(),
                            is_enabled: service.is_enabled.clone(),
                            pid: -1,
                            logs: String::new(),
                        }))
                    })
                    .collect(),
                cursor: 0,
            })
            .collect();
        app.key_bindings = config.keybindings.clone();
        app.service_groups = service_groups;
        Self::save_config(app)
    }

    /// Saves the configuration to the configuration file.
    /// 
    /// # Arguments
    /// 
    /// * `app` - The application instance to save the configuration from.
    pub fn save_config(app: &App) -> io::Result<()> {
        let groups: Vec<ConfigGroup> = app
            .service_groups
            .iter()
            .map(|group| ConfigGroup {
                name: group.name.clone(),
                is_active: group.is_active.clone(),
                is_enabled: group.is_enabled.clone(),
                services: group
                    .services
                    .iter()
                    .map(|service| {
                        let service = service.lock().unwrap();
                        ConfigService {
                            name: service.name.clone(),
                            description: service.description.clone(),
                            path: service.path.clone(),
                            is_active: service.is_active.clone(),
                            is_enabled: service.is_enabled.clone(),
                        }
                    })
                    .collect(),
            })
            .collect();
        let config = Config {
            keybindings: app.key_bindings.clone(),
            groups,
        };
        let config_path = Self::get_config_path();
        std::fs::write(&config_path, toml::to_string(&config).unwrap())
    }
}
