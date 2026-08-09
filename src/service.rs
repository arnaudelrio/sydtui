use std::collections::HashMap;
use std::io;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};

use ratatui::style::{Color, Style};
use ratatui::text::Span;
use ratatui::{
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph},
};
use serde::{Deserialize, Serialize};

/// Represents a service on the system.
#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct Service {
    /// The name of the service.
    pub name: String,
    /// The description of the service.
    pub description: String,
    /// The path to the service file.
    pub path: PathBuf,
    /// Whether the service is active.
    pub is_active: bool,
    /// Whether the service is enabled.
    pub is_enabled: bool,
    /// The process ID of the service.
    pub pid: i32,
    /// The logs of the service.
    pub logs: String,
}

/// Represents a summary of a unit on the system.
#[derive(Debug, Default, Deserialize, Serialize, Clone)]
struct UnitSummary {
    /// The name of the unit.
    unit: String,
    /// The description of the unit.
    description: String,
    /// Whether the unit is active.
    active: String,
}

impl Service {
    /// Returns all services on the system.
    pub fn get_all() -> Vec<Arc<Mutex<Self>>> {
        let output = Command::new("systemctl")
            .args(["list-units", "--type=service", "--all", "--output=json"])
            .output();

        let output = match output {
            Ok(o) if o.status.success() => o.stdout,
            _ => return vec![],
        };

        let summaries: Vec<UnitSummary> = serde_json::from_slice(&output).unwrap_or_default();
        if summaries.is_empty() {
            return vec![];
        }

        let unit_names: Vec<&str> = summaries.iter().map(|s| s.unit.as_str()).collect();
        let properties_map = Self::fetch_unit_properties(&unit_names);
        summaries
            .into_iter()
            .map(|s| {
                let props = properties_map.get(&s.unit);

                let path = props
                    .and_then(|p| p.get("FragmentPath"))
                    .map(PathBuf::from)
                    .unwrap_or_default();

                let pid = props
                    .and_then(|p| p.get("MainPID"))
                    .and_then(|p| p.parse::<i32>().ok())
                    .unwrap_or(0);

                let is_enabled = props
                    .and_then(|p| p.get("UnitFileState"))
                    .map(|state| state == "enabled" || state == "enabled-runtime")
                    .unwrap_or(false);

                // Fetch recent logs (e.g., last 5 lines)
                let logs = Self::fetch_logs(&s.unit, 10);

                Arc::new(Mutex::new(Service {
                    name: s.unit,
                    description: s.description,
                    path,
                    is_active: s.active == "active",
                    is_enabled,
                    pid,
                    logs,
                }))
            })
            .collect()
    }

    /// Bulk queries properties for multiple unit files using systemctl show
    fn fetch_unit_properties(units: &[&str]) -> HashMap<String, HashMap<String, String>> {
        let mut map = HashMap::new();
        if units.is_empty() {
            return map;
        }

        let mut cmd = Command::new("systemctl");
        cmd.arg("show")
            .arg("--property=Id,FragmentPath,MainPID,UnitFileState");

        for unit in units {
            cmd.arg(unit);
        }

        let output = match cmd.output() {
            Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).into_owned(),
            _ => return map,
        };

        // systemctl show outputs blocks separated by empty lines
        for block in output.split("\n\n") {
            let mut props = HashMap::new();
            for line in block.lines() {
                if let Some((k, v)) = line.split_once('=') {
                    props.insert(k.to_string(), v.to_string());
                }
            }
            if let Some(id) = props.get("Id").cloned() {
                map.insert(id, props);
            }
        }

        map
    }

    /// Queries the journal for recent log lines of a unit
    fn fetch_logs(unit: &str, lines: usize) -> String {
        let output = Command::new("journalctl")
            .args(["-u", unit, "-n", &lines.to_string(), "--no-pager", "-q"])
            .output();

        match output {
            Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
            Err(_) => String::new(),
        }
    }

    /// Gets the logs of the service.
    pub fn get_logs(&mut self) {
        self.logs = Service::fetch_logs(&self.name, 100);
    }

    /// Gets a service by its name.
    pub fn get_service_by_name(
        app_services: &Vec<Arc<Mutex<Service>>>,
        service_name: String,
    ) -> Option<&Arc<Mutex<Service>>> {
        let service = Service::search_services(app_services, service_name, 1)
            .unwrap_or_default()
            .first()?
            .to_owned();
        app_services
            .iter()
            .find(|s| s.lock().unwrap().name == service)
    }

    /// Draws the list item for the service.
    pub fn draw_list(&self, is_selected: bool) -> Paragraph<'_> {
        let active = if self.is_active { "active" } else { "inactive" };
        let enabled = if self.is_enabled { "enabled" } else { "disabled" };
        let path = if self.path.to_string_lossy().is_empty() { String::new() } else { format!(" \"{}\"", self.path.to_string_lossy()) };
        let description = if self.description.is_empty() { String::new() } else { format!(" ({})", self.description) };
        let block = Block::bordered().border_set(border::THICK);
        let style = if is_selected {
            Style::default().fg(Color::Black).bg(Color::Green)
        } else {
            Style::default().fg(Color::Cyan)
        };
        let service_name = Line::from(vec![
            Span::raw(" "),
            Span::styled(self.name.clone(), Style::default().underlined()),
            Span::raw(format!("{}{}", path, description)),
        ]);
        let service_properties = Line::from(format!("  {} - {}", active, enabled));

        Paragraph::new(Text::from(vec![service_name, service_properties]))
            .style(style)
            .block(block)
    }

    /// Searches for services by name.
    pub fn search_services(
        service_list: &Vec<Arc<Mutex<Service>>>,
        name: String,
        count: usize,
    ) -> io::Result<Vec<String>> {
        let mut services: Vec<String> = service_list
            .iter()
            .map(|s| s.lock().unwrap().name.clone())
            .filter(|s| s.to_lowercase().contains(name.to_lowercase().as_str()))
            .collect::<Vec<String>>();
        services.sort_by(|a, b| {
            let idx_a = a.to_lowercase().find(name.to_lowercase().as_str()).unwrap();
            let idx_b = b.to_lowercase().find(name.to_lowercase().as_str()).unwrap();
            idx_a
                .cmp(&idx_b)
                .then_with(|| a.to_lowercase().cmp(&b.to_lowercase()))
        });
        if services.len() >= count {
            Ok(services[0..count].to_vec())
        } else {
            Ok(services)
        }
    }

    /// Starts the service.
    pub fn start_service(&mut self) -> io::Result<()> {
        self.is_active = true;
        Command::new("systemctl")
            .arg("start")
            .arg(self.name.clone())
            .output()?;
        Ok(())
    }

    /// Stops the service.
    pub fn stop_service(&mut self) -> io::Result<()> {
        self.is_active = false;
        Command::new("systemctl")
            .arg("stop")
            .arg(self.name.clone())
            .output()?;
        Ok(())
    }

    /// Enables the service.
    pub fn enable_service(&mut self) -> io::Result<()> {
        self.is_enabled = true;
        Command::new("systemctl")
            .arg("enable")
            .arg(self.name.clone())
            .output()?;
        Ok(())
    }

    /// Disables the service.
    pub fn disable_service(&mut self) -> io::Result<()> {
        self.is_enabled = false;
        Command::new("systemctl")
            .arg("disable")
            .arg(self.name.clone())
            .output()?;
        Ok(())
    }
}
