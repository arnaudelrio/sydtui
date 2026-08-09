use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};

use crate::app::{App, CurrentScreen};

/// Handles key presses for the application.
pub fn handle_events(app: &mut App) -> io::Result<()> {
    if let Event::Key(key_event) = event::read()? {
        if key_event.kind == KeyEventKind::Press {
            match app.current_screen.clone() {
                CurrentScreen::MainMenu => match key_event.code {
                    key if app.key_bindings.exit.contains(&key) => app.exit(),
                    key if app.key_bindings.help.contains(&key) => app.help(),
                    key if app.key_bindings.enter.contains(&key)
                        && !app.service_groups.is_empty() =>
                    {
                        app.select_group()
                    }
                    key if app.key_bindings.save.contains(&key) => app.save()?,
                    key if app.key_bindings.reload.contains(&key) => app.reload()?,
                    key if app.key_bindings.previous.contains(&key)
                        && !app.service_groups.is_empty() =>
                    {
                        app.previous_group()
                    }
                    key if app.key_bindings.next.contains(&key)
                        && !app.service_groups.is_empty() =>
                    {
                        app.next_group()
                    }
                    key if app.key_bindings.new.contains(&key) => {
                        app.current_screen = CurrentScreen::CreatingNew {
                            text: String::new(),
                        }
                    }
                    key if app.key_bindings.rename.contains(&key)
                        && !app.service_groups.is_empty() =>
                    {
                        app.rename_group();
                        app.save()?;
                    }
                    
                    key if app.key_bindings.duplicate.contains(&key)
                        && !app.service_groups.is_empty() =>
                    {
                        app.duplicate_group();
                        app.save()?;
                    }
                    key if app.key_bindings.delete.contains(&key)
                        && !app.service_groups.is_empty() =>
                    {
                        app.delete_group();
                        app.save()?;
                    }
                    key if app.key_bindings.toggle_activate.contains(&key)
                        && !app.service_groups.is_empty() =>
                    {
                        app.toggle_activate(None)?;
                        app.save()?;
                    }
                    key if app.key_bindings.toggle_enabled.contains(&key)
                        && !app.service_groups.is_empty() =>
                    {
                        app.toggle_enabled(None)?;
                        app.save()?;
                    }
                    _ => {}
                },
                CurrentScreen::KeyBindings { previous_screen } => {
                    if app.key_bindings.exit.contains(&key_event.code) {
                        app.current_screen = *previous_screen;
                    }
                }
                CurrentScreen::CreatingNew { text } => match key_event.code {
                    key if app.key_bindings.exit.contains(&key) => {
                        app.current_screen = CurrentScreen::MainMenu
                    }
                    key if app.key_bindings.enter.contains(&key) => {
                        app.submit_new(text.clone());
                        app.save()?;
                    }
                    KeyCode::Backspace => {
                        if text.len() != 0 {
                            app.current_screen = CurrentScreen::CreatingNew {
                                text: format!(
                                    "{}",
                                    text.chars().take(text.len() - 1).collect::<String>()
                                ),
                            }
                        } else {
                            app.current_screen = CurrentScreen::MainMenu
                        }
                    }
                    KeyCode::Char(value) => {
                        app.current_screen = CurrentScreen::CreatingNew {
                            text: format!("{}{}", text.clone(), value),
                        };
                    }
                    _ => {}
                },
                CurrentScreen::RenameGroup { text } => match key_event.code {
                    key if app.key_bindings.exit.contains(&key) => {
                        app.current_screen = CurrentScreen::MainMenu
                    }
                    key if app.key_bindings.enter.contains(&key) => {
                        app.submit_rename(text.clone());
                        app.save()?;
                    }
                    KeyCode::Backspace => {
                        if text.len() != 0 {
                            app.current_screen = CurrentScreen::RenameGroup {
                                text: format!(
                                    "{}",
                                    text.chars().take(text.len() - 1).collect::<String>()
                                ),
                            }
                        } else {
                            app.current_screen = CurrentScreen::MainMenu
                        }
                    }
                    KeyCode::Char(value) => {
                        app.current_screen = CurrentScreen::RenameGroup {
                            text: format!("{}{}", text.clone(), value),
                        };
                    }
                    _ => {}
                }
                CurrentScreen::GroupView { index } => {
                    let group = &mut app.service_groups[app.cursor];
                    group.refresh();
                    match key_event.code {
                        key if app.key_bindings.exit.contains(&key) => {
                            app.current_screen = CurrentScreen::MainMenu
                        }
                        key if app.key_bindings.help.contains(&key) => {
                            app.help();
                        }
                        key if app.key_bindings.save.contains(&key) => {
                            app.save()?;
                        }
                        key if app.key_bindings.previous.contains(&key)
                            && !group.services.is_empty() =>
                        {
                            group.previous_service();
                            let _ = &group.services[group.cursor].lock().unwrap().get_logs();
                        }
                        key if app.key_bindings.next.contains(&key)
                            && !group.services.is_empty() =>
                        {
                            group.next_service();
                            let _ = &group.services[group.cursor].lock().unwrap().get_logs();
                        }
                        key if app.key_bindings.new.contains(&key) => {
                            app.current_screen = CurrentScreen::ServiceSelection {
                                group_index: app.cursor,
                                text: String::new(),
                            }
                        }
                        key if app.key_bindings.delete.contains(&key) => {
                            group.delete_service(index);
                            group.refresh();
                            app.save()?;
                        }
                        key if app.key_bindings.toggle_activate.contains(&key)
                            && !group.services.is_empty() =>
                        {
                            group.toggle_service_active()?;
                            group.refresh();
                            app.save()?;
                        }
                        key if app.key_bindings.toggle_enabled.contains(&key)
                            && !group.services.is_empty() =>
                        {
                            group.toggle_service_enabled()?;
                            group.refresh();
                            app.save()?;
                        }
                        key if app.key_bindings.view_logs.contains(&key)
                            && !group.services.is_empty() =>
                        {
                            let _ = &group.services[index].lock().unwrap().get_logs();
                            app.current_screen = CurrentScreen::ServiceLogs {
                                group_index: app.cursor,
                                index: group.cursor,
                                log_scroll: 0,
                            };
                        }
                        _ => {}
                    };
                }
                CurrentScreen::ServiceSelection { group_index, text } => match key_event.code {
                    key if app.key_bindings.exit.contains(&key) => {
                        app.current_screen = CurrentScreen::GroupView { index: group_index };
                    }
                    key if app.key_bindings.enter.contains(&key) => {
                        if let Some(group) = app.service_groups.get_mut(group_index) {
                            group.add_service(&app.services, text);
                            app.current_screen = CurrentScreen::GroupView { index: group_index };
                            app.save()?;
                        }
                    }
                    KeyCode::Backspace => {
                        if text.len() != 0 {
                            app.current_screen = CurrentScreen::ServiceSelection {
                                group_index: group_index,
                                text: format!(
                                    "{}",
                                    text.chars().take(text.len() - 1).collect::<String>()
                                ),
                            }
                        } else {
                            app.current_screen = CurrentScreen::GroupView { index: group_index }
                        }
                    }
                    KeyCode::Char(value) => {
                        app.current_screen = CurrentScreen::ServiceSelection {
                            group_index: group_index,
                            text: format!("{}{}", text.clone(), value),
                        };
                    }
                    _ => {}
                },
                CurrentScreen::ServiceLogs {
                    group_index,
                    index,
                    log_scroll,
                } => {
                    let group = &app.service_groups[group_index];
                    let service = &group.services[index].lock().unwrap();

                    match key_event.code {
                        key if app.key_bindings.previous.contains(&key) => {
                            if log_scroll < service.logs.lines().count() - 1 {
                                app.current_screen = CurrentScreen::ServiceLogs {
                                    group_index,
                                    index,
                                    log_scroll: log_scroll + 1,
                                };
                            }
                        }
                        key if app.key_bindings.next.contains(&key) => {
                            if log_scroll > 0 {
                                app.current_screen = CurrentScreen::ServiceLogs {
                                    group_index,
                                    index,
                                    log_scroll: log_scroll - 1,
                                };
                            }
                        }
                        key if app.key_bindings.exit.contains(&key) => {
                            app.current_screen = CurrentScreen::GroupView { index: group_index };
                        }
                        _ => {}
                    };
                }
            }
        }
    };
    Ok(())
}
