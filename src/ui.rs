use std::io;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Margin},
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Clear, Paragraph},
};

use crate::{app::{App, CurrentScreen}, service::Service, service_groups::ServiceGroup};

/// Renders the UI for the application. Redirects to the appropriate screen based on the [`CurrentScreen`] state.
pub fn ui(frame: &mut Frame, app: &mut App) {
    match &app.current_screen {
        CurrentScreen::MainMenu => render_main_menu(frame, app),
        CurrentScreen::KeyBindings { previous_screen } => {
            match &**previous_screen {
                CurrentScreen::MainMenu => render_main_menu(frame, app),
                CurrentScreen::GroupView { index } => {
                    render_group_view(frame, app, &app.service_groups[*index])
                }
                _ => {}
            }
            render_key_bindings(frame, app);
        }
        CurrentScreen::CreatingNew { text } => render_creating_new(frame, app, text.clone()),
        CurrentScreen::RenameGroup { text } => render_rename_group(frame, app, text.clone()),
        CurrentScreen::GroupView { index } => {
            render_group_view(frame, app, &app.service_groups[*index])
        }
        CurrentScreen::ServiceSelection { group_index, text } => {
            let service_group = &app.service_groups[*group_index];
            let _ = render_service_selection(frame, app, service_group, text.clone());
        }
        CurrentScreen::ServiceLogs {
            group_index,
            index,
            log_scroll,
        } => {
            let service_group = &app.service_groups[*group_index];
            render_service_logs(frame, app, service_group, *index, *log_scroll)
        }
    }
}

fn render_main_menu(frame: &mut Frame, app: &mut App) {
    let title = Line::from(" sydtui: systemd services manager ".bold());
    let keybindings = Line::from(vec![
        " New group ".into(),
        "<n>".blue().bold(),
        " Navigate groups ".into(),
        "<Up> <Down>".blue().bold(),
        " Select group ".into(),
        "<Enter>".blue().bold(),
        " Help ".into(),
        "<?>".blue().bold(),
        " Quit ".into(),
        "<q> ".blue().bold(),
    ]);
    let block = Block::bordered()
        .title(title.centered())
        .title_bottom(keybindings.centered())
        .border_set(border::THICK);

    let area = block.inner(frame.area());
    frame.render_widget(block, frame.area());

    let item_height = 3;
    let visible_capacity = (area.height / item_height) as usize;
    let scroll_offset = if app.cursor >= visible_capacity {
        app.cursor + 1 - visible_capacity
    } else {
        0
    };
    let extra = if visible_capacity != area.height.div_ceil(item_height) as usize || scroll_offset + 1 + visible_capacity > app.service_groups.len() {
        1
    } else {
        0
    };

    let visible_groups = &app
        .service_groups
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(visible_capacity + extra)
        .collect::<Vec<_>>();

    let mut constraints = vec![Constraint::Length(item_height); visible_groups.len().max(1) - 1];
    if visible_groups.len() == visible_capacity + 1 {
        constraints.push(Constraint::Min(1));
    } else if extra == 0 || scroll_offset + 1 + visible_capacity >= app.service_groups.len() {
        constraints.push(Constraint::Length(item_height));
    }
    

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    for ((i, group), chunk) in visible_groups.iter().zip(chunks.iter()) {
        frame.render_widget(
            group.draw_list(*i == app.cursor),
            chunk.inner(Margin::new(1, 0)),
        );
    }
}

fn render_key_bindings(frame: &mut Frame, app: &App) {
    let title = Line::from(" Key bindings ".bold());
    let block = Block::bordered().title(title).border_set(border::THICK);
    let text = Text::from(app.key_bindings.print_help());

    let area = frame
        .area()
        .clone()
        .centered(Constraint::Length(50), Constraint::Length(16));

    frame.render_widget(Clear, area.outer(Margin::new(1, 0)));
    frame.render_widget(Paragraph::new(text).block(block), area);
}

fn render_creating_new(frame: &mut Frame, app: &mut App, text: String) {
    render_main_menu(frame, app);

    let title = Line::from(" Creating new group ".bold());
    let block = Block::bordered().title(title).border_set(border::THICK);
    let text = Text::from(format!(" {}", text));

    // Clear space for popup
    let area = frame
        .area()
        .clone()
        .centered(Constraint::Length(50), Constraint::Length(3));

    frame.render_widget(Clear, area.outer(Margin::new(1, 0)));
    frame.render_widget(Paragraph::new(text).block(block), area);
}

fn render_rename_group(frame: &mut Frame, app: &mut App, text: String) {
    render_main_menu(frame, app);

    let title = Line::from(" Rename group ".bold());
    let block = Block::bordered().title(title).border_set(border::THICK);
    let text = Text::from(format!(" {}", text));
    
    let area = frame
        .area()
        .clone()
        .centered(Constraint::Length(50), Constraint::Length(3));

    frame.render_widget(Clear, area.outer(Margin::new(1, 0)));
    frame.render_widget(Paragraph::new(text).block(block), area);
}

fn render_group_view(frame: &mut Frame, app: &App, service_group: &ServiceGroup) {
    let title = Line::from(" sydtui: systemd services manager ".bold());
    let keybindings = Line::from(vec![
        " Add service ".into(),
        "<n>".blue().bold(),
        " Navigate services ".into(),
        "<Up> <Down>".blue().bold(),
        " Help ".into(),
        "<?>".blue().bold(),
        " Quit ".into(),
        "<q> ".blue().bold(),
    ]);
    let block = Block::bordered()
        .title(title.centered())
        .title_bottom(keybindings.centered())
        .border_set(border::THICK);

    let main_area = block.inner(frame.area());
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Length(4), Constraint::Min(4)])
        .split(main_area);
    frame.render_widget(block, frame.area());

    let header = service_group.draw_header(app);
    frame.render_widget(header, main_layout[0].inner(Margin::new(1, 0)));

    let list_area = main_layout[1];
    let item_height = 4;
    let visible_capacity = (list_area.height / item_height) as usize;
    let scroll_offset = if service_group.cursor >= visible_capacity {
        service_group.cursor + 1 - visible_capacity
    } else {
        0
    };
    let extra = if visible_capacity != list_area.height.div_ceil(item_height) as usize || scroll_offset + 1 + visible_capacity > service_group.services.len() {
        1
    } else {
        0
    };

    let visible_services = service_group
        .services
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(visible_capacity + extra)
        .collect::<Vec<_>>();

    let mut constraints = vec![Constraint::Length(item_height); visible_services.len().max(1) - 1];
    if visible_services.len() == visible_capacity + 1 {
        constraints.push(Constraint::Min(1));
    } else if extra == 0 || scroll_offset + 1 + visible_capacity >= service_group.services.len() {
        constraints.push(Constraint::Length(item_height));
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(list_area);

    // 5. Render visible chunks
    for ((i, service), chunk) in visible_services.iter().zip(chunks.iter()) {
        frame.render_widget(
            service.lock().unwrap().draw_list(*i == service_group.cursor),
            chunk.inner(Margin::new(2, 0)),
        );
    }
}

fn render_service_selection(
    frame: &mut Frame,
    app: &App,
    service_group: &ServiceGroup,
    text: String,
) -> io::Result<()> {
    render_group_view(frame, app, service_group);

    let title = Line::from(" Adding new service to group ".bold());
    let block_outer = Block::bordered().title(title).border_set(border::THICK);
    let input_text_field = Text::from(format!(" {} ", text));

    let area = frame
        .area()
        .clone()
        .centered(Constraint::Percentage(50), Constraint::Percentage(80));

    let main_area = block_outer.inner(area);
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Length(3), Constraint::Min(3)])
        .split(main_area);
    frame.render_widget(Clear, area.outer(Margin::new(1, 0)));
    frame.render_widget(block_outer, area);

    let block = Block::bordered().border_set(border::THICK);
    frame.render_widget(
        Paragraph::new(input_text_field).block(block),
        main_layout[0].inner(Margin::new(1, 0)),
    );

    let list_area = main_layout[1];
    let item_height = 3;
    let visible_capacity = (list_area.height / item_height) as usize;
    let services = Service::search_services(&app.services, text, visible_capacity)?;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Length(item_height); services.len()])
        .split(list_area);

    for (service, chunk) in services.iter().zip(chunks.iter()) {
        let block = Block::bordered().border_set(border::THICK);
        frame.render_widget(
            Paragraph::new(Text::from(format!(" {} ", service))).block(block),
            chunk.inner(Margin::new(1, 0)),
        );
    }
    Ok(())
}

fn render_service_logs(
    frame: &mut Frame,
    app: &App,
    service_group: &ServiceGroup,
    service_index: usize,
    log_scroll: usize,
) {
    render_group_view(frame, app, service_group);

    let service = service_group.services[service_index].lock().unwrap();

    let title = Line::from(format!(" Service Logs for {} ", service.name).bold());
    let block = Block::bordered().title(title).border_set(border::THICK);

    // Clear space for popup
    let area = frame
        .area()
        .clone()
        .centered(Constraint::Percentage(80), Constraint::Percentage(80));

    let inner_area = block.inner(area);
    let log_lines: Vec<Line> = service.logs.lines().map(|line| Line::from(line)).collect();

    // Calculate visible height
    let visible_height = inner_area.height as usize;
    let total_lines = log_lines.len();

    // Calculate which lines to show (reversed scrolling: log_scroll=0 shows bottom)
    let start_index = if total_lines > visible_height {
        (total_lines - visible_height).saturating_sub(log_scroll)
    } else {
        0
    };

    let end_index = (start_index + visible_height).min(total_lines);
    let highlight_index = if total_lines == 0 {
        0
    } else if total_lines > visible_height {
        total_lines - 1 - log_scroll
    } else {
        total_lines - 1
    };

    // Build the displayed lines with highlighting
    let displayed_lines: Vec<Line> = log_lines[start_index..end_index]
        .iter()
        .enumerate()
        .map(|(i, line)| {
            let absolute_index = start_index + i;
            if absolute_index == highlight_index {
                Line::from(line.clone()).reversed()
            } else {
                line.clone()
            }
        })
        .collect();

    let logs = Paragraph::new(displayed_lines);

    frame.render_widget(Clear, area.outer(Margin::new(1, 0)));
    frame.render_widget(logs.block(block), area);
}
