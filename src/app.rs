use std::io;
use std::sync::{Arc, Mutex};

use ratatui::DefaultTerminal;
use serde::Deserialize;

use crate::config::Config;
use crate::events;
use crate::key_bindings::Keybindings;
use crate::service::Service;
use crate::service_groups::ServiceGroup;
use crate::ui::ui;

/// The main application struct for the sydtui application.
/// 
/// This struct holds the state of the application, including the service groups, cursor position, current screen, services, key bindings, and exit flag.
#[derive(Debug, Deserialize)]
pub struct App {
    /// The list of service groups.
    pub service_groups: Vec<ServiceGroup>,
    /// The index of the currently selected service group.
    pub cursor: usize,
    /// The current screen of the application.
    pub current_screen: CurrentScreen,
    /// The full list of available services.
    pub services: Vec<Arc<Mutex<Service>>>,
    /// The key bindings for the application.
    pub key_bindings: Keybindings,
    /// Whether the application should exit.
    pub exit: bool,
}

/// The current screen of the application.
#[derive(Debug, Deserialize, PartialEq, Clone)]
pub enum CurrentScreen {
    /// The main menu screen. Shows the list of service groups.
    MainMenu,
    /// The key bindings popup.
    KeyBindings { previous_screen: Box<CurrentScreen> },
    /// The creating new popup.
    CreatingNew { text: String },
    /// The rename group popup.
    RenameGroup { text: String },
    /// The group view screen.
    GroupView { index: usize },
    /// The service selection popup.
    ServiceSelection { group_index: usize, text: String },
    /// The service logs popup.
    ServiceLogs { group_index: usize, index: usize, log_scroll: usize },
}

impl App {
    /// Initializes a new instance of the application and loads the configuration.
    pub fn init() -> io::Result<Self> {
        let mut app = App {
            service_groups: vec![],
            cursor: 0,
            current_screen: CurrentScreen::MainMenu,
            services: Service::get_all(),
            key_bindings: Keybindings::default(),
            exit: false,
        };
        Config::load_config(&mut app)?;
        Ok(app)
    }

    /// Runs the main event loop of the application for the TUI.
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        terminal.clear()?;

        while !self.exit {
            terminal.draw(|frame| ui(frame, self))?;
            events::handle_events(self)?;
        }
        Ok(())
    }
    
    /// Displays the help screen.
    pub fn help(&mut self) {
        self.current_screen = CurrentScreen::KeyBindings {
            previous_screen: Box::new(self.current_screen.clone()),
        };
    }

    /// Submits a new service group with the given text.
    pub fn submit_new(&mut self, text: String) {
        let service_group = ServiceGroup::new(text);
        self.service_groups.push(service_group);
        self.cursor = self.service_groups.len() - 1;
        self.current_screen = CurrentScreen::MainMenu;
    }

    /// Renames the current service group.
    pub fn rename_group(&mut self) {
        let group_name = self.service_groups[self.cursor].name.clone();
        self.current_screen = CurrentScreen::RenameGroup { text: group_name };
    }

    /// Submits a renamed service group with the given text.
    pub fn submit_rename(&mut self, text: String) {
        self.service_groups[self.cursor].name = text;
        self.current_screen = CurrentScreen::MainMenu;
    }

    /// Duplicates the current service group.
    pub fn duplicate_group(&mut self) {
        let group = self.service_groups[self.cursor].clone();
        let name = group.name.clone();
        self.service_groups.push(group);
        self.cursor = self.service_groups.len() - 1;
        self.current_screen = CurrentScreen::RenameGroup { text: name };
    }

    /// Saves the configuration to disk.
    pub fn save(&mut self) -> io::Result<()> {
        Config::save_config(self)
    }

    /// Deletes the current service group.
    pub fn delete_group(&mut self) {
        self.service_groups.remove(self.cursor);

        if self.service_groups.len() != 0 && self.cursor >= self.service_groups.len() {
            self.cursor = self.service_groups.len() - 1;
        }
    }

    /// Exits the application.
    pub fn exit(&mut self) {
        self.exit = true;
    }

    /// Reloads the configuration from disk.
    pub fn reload(&mut self) -> io::Result<()> {
        Config::load_config(self)
    }

    /// Lists all service groups.
    pub fn list_service_groups(&mut self) {
        for (index, group) in self.service_groups.iter().enumerate() {
            print!("{}: {}\n", index+1, group.name);
        }
    }

    /// Moves the cursor to the next group.
    pub fn next_group(&mut self) {
        if self.cursor == self.service_groups.len() - 1 {
            self.cursor = 0;
        } else {
            self.cursor += 1;
        }
    }

    /// Moves the cursor to the previous group.
    pub fn previous_group(&mut self) {
        if self.cursor == 0 {
            self.cursor = self.service_groups.len() - 1;
        } else {
            self.cursor -= 1;
        }
    }

    /// Selects the current group.
    pub fn select_group(&mut self) {
        self.current_screen = CurrentScreen::GroupView { index: self.cursor };
    }

    /// Toggles the activation state of the current group.
    /// 
    /// If a name is provided, the cursor is moved to the group with that name (useful for CLI-mode). Case-insensitive.
    pub fn toggle_activate(&mut self, name: Option<String>) -> io::Result<bool> {
        if let Some(name) = name {
            for (index, group) in self.service_groups.iter_mut().enumerate() {
                if group.name == name.to_lowercase() {
                    self.cursor = index;
                    break;
                }
            }
        }
        if let Some(group) = self.service_groups.get_mut(self.cursor) {
            group.toggle_active()
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No service group found",
            ))
        }
    }

    /// Toggles the enabled state of the current group.
    /// 
    /// If a name is provided, the cursor is moved to the group with that name (useful for CLI-mode). Case-insensitive.
    pub fn toggle_enabled(&mut self, name: Option<String>) -> io::Result<bool> {
        if let Some(name) = name {
            for (index, group) in self.service_groups.iter_mut().enumerate() {
                if group.name == name.to_lowercase() {
                    self.cursor = index;
                    break;
                }
            }
        }
        if let Some(group) = self.service_groups.get_mut(self.cursor) {
            group.toggle_enabled()
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No service group found",
            ))
        }
    }
}