use std::sync::Arc;
use std::{io, sync::Mutex};

use ratatui::style::Stylize;
use ratatui::text::Span;
use ratatui::{
    style::{Color, Style},
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph},
};
use serde::{Deserialize, Serialize};

use crate::{App, service::Service};

/// Represents a group of services.
#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct ServiceGroup {
    /// The name of the service group.
    pub name: String,
    /// Whether the service group is active.
    pub is_active: bool,
    /// Whether the service group is enabled.
    pub is_enabled: bool,
    /// The services in the group.
    pub services: Vec<Arc<Mutex<Service>>>,
    /// The index of the currently selected service.
    pub cursor: usize,
}

impl ServiceGroup {
    /// Draws a list item for the service group.
    pub fn draw_list(&self, is_selected: bool) -> Paragraph<'_> {
        let active = if self.is_active { "active" } else { "inactive" };
        let enabled = if self.is_enabled { "enabled" } else { "disabled" };
        let block = Block::bordered().border_set(border::THICK);
        let style = if is_selected {
            Style::default().fg(Color::Black).bg(Color::Green)
        } else {
            Style::default().fg(Color::Cyan)
        };
        let group_text = Line::from(vec![
            Span::raw(" "),
            Span::styled(self.name.clone(), Style::default().bold()),
            Span::raw(format!(" ({} - {})", active, enabled)),
        ]);

        Paragraph::new(group_text).style(style).block(block)
    }

    /// Draws the header for the service group view.
    pub fn draw_header(&self, app: &App) -> Paragraph<'_> {
        let active = if self.is_active { "active" } else { "inactive" };
        let enabled = if self.is_enabled { "enabled" } else { "disabled" };
        let block = Block::bordered().border_set(border::THICK);

        let group_name = Line::from(vec![
            Span::from(format!(" {}. Service group: ", app.cursor + 1)),
            Span::from(format!("{}", self.name)).bold(),
        ]);
        let group_properties = Line::from(format!(" {} - {}", active, enabled));

        Paragraph::new(Text::from(vec![group_name, group_properties])).block(block)
    }

    /// Creates a new service group with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        ServiceGroup {
            name: name.into(),
            is_active: false,
            is_enabled: false,
            services: vec![],
            cursor: 0,
        }
    }

    /// Refreshes the active and enabled status of the service group.
    pub fn refresh(&mut self) {
        if self.services.iter().filter(|s| !s.lock().unwrap().is_active).collect::<Vec<&Arc<Mutex<Service>>>>().is_empty() {
            self.is_active = true;
        } else {
            self.is_active = false;
        }

        if self.services.iter().filter(|s| !s.lock().unwrap().is_enabled).collect::<Vec<&Arc<Mutex<Service>>>>().is_empty() {
            self.is_enabled = true;
        } else {
            self.is_enabled = false;
        }
    }

    /// Adds a service to the group based on the given text.
    pub fn add_service(&mut self, app_services: &Vec<Arc<Mutex<Service>>>, text: String) {
        let get_service = Service::get_service_by_name(app_services, text).cloned();

        if let Some(service) = get_service {
            if !self
                .services
                .iter()
                .any(|s| s.lock().unwrap().name == service.lock().unwrap().name)
            {
                self.services.push(service);
            }
            return;
        }
    }

    /// Deletes a service from the group at the given index.
    pub fn delete_service(&mut self, index: usize) {
        self.services.remove(index);
        if self.services.len() != 0 && self.cursor >= self.services.len() {
            self.cursor = self.services.len() - 1;
        }
    }

    /// Moves the cursor to the next service in the group.
    pub fn next_service(&mut self) {
        if self.cursor == self.services.len() - 1 {
            self.cursor = 0;
        } else {
            self.cursor += 1;
        }
    }

    /// Moves the cursor to the previous service in the group.
    pub fn previous_service(&mut self) {
        if self.cursor == 0 {
            self.cursor = self.services.len() - 1;
        } else {
            self.cursor -= 1;
        }
    }

    /// Starts all services in the group.
    pub fn start_services(&mut self) -> io::Result<()> {
        for services in &self.services {
            services.lock().unwrap().start_service()?;
        }
        self.is_active = true;
        Ok(())
    }

    /// Stops all services in the group.
    pub fn stop_services(&mut self) -> io::Result<()> {
        for services in &self.services {
            services.lock().unwrap().stop_service()?;
        }
        self.is_active = false;
        Ok(())
    }

    /// Toggles the active state of the group.
    pub fn toggle_active(&mut self) -> io::Result<bool> {
        if self.is_active {
            self.stop_services()?;
            Ok(false)
        } else {
            self.start_services()?;
            Ok(true)
        }
    }

    /// Enables all services in the group.
    pub fn enable_services(&mut self) -> io::Result<()> {
        for services in &self.services {
            services.lock().unwrap().enable_service()?;
        }
        self.is_enabled = true;
        Ok(())
    }

    /// Disables all services in the group.
    pub fn disable_services(&mut self) -> io::Result<()> {
        for services in &self.services {
            services.lock().unwrap().disable_service()?;
        }
        self.is_enabled = false;
        Ok(())
    }

    /// Toggles the enabled state of the group.
    pub fn toggle_enabled(&mut self) -> io::Result<bool> {
        if self.is_enabled {
            self.disable_services()?;
            Ok(false)
        } else {
            self.enable_services()?;
            Ok(true)
        }
    }

    /// Toggles the active state of the group.
    pub fn toggle_service_active(&mut self) -> io::Result<()> {
        let mut service = self.services[self.cursor].lock().unwrap();

        if service.is_active {
            service.stop_service()
        } else {
            service.start_service()
        }
    }

    /// Toggles the enabled state of the group.
    pub fn toggle_service_enabled(&mut self) -> io::Result<()> {
        let mut service = self.services[self.cursor].lock().unwrap();

        if service.is_enabled {
            service.disable_service()
        } else {
            service.enable_service()
        }
    }
}
