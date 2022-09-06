use tui::style::*;
use tui::widgets::*;

use std::io;

mod config;
mod model;

fn get_dir_entries(path: &std::path::Path) -> Vec<std::fs::DirEntry> {
    match std::fs::read_dir(path) {
        Ok(entries) => entries
            .filter_map(|result| match result {
                Ok(entry) => Some(entry),
                Err(error) => {
                    eprintln!("{error}");
                    None
                }
            })
            .collect(),
        Err(error) => {
            eprintln!("{error}");
            Vec::new()
        }
    }
}

fn main() -> Result<(), io::Error> {
    let mut options = config::read_config(std::path::Path::new("config.lua"));
    let stdout = io::stdout();
    let backend = tui::backend::CrosstermBackend::new(stdout);
    let mut terminal = tui::terminal::Terminal::new(backend)?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::EnterAlternateScreen
    )?;
    crossterm::terminal::enable_raw_mode()?;

    let mut selected_index = 0;

    let cwd = std::path::Path::new("/home/midwest");
    let mut dir_entries = get_dir_entries(&cwd);

    loop {
        // -- draw
        let style_selection = Style::default().fg(Color::Black).bg(Color::LightYellow);

        let mut state = TableState::default();
        state.select(Some(selected_index));

        use tui::layout::Constraint;
        terminal.draw(|f| {
            let size = f.size();
            let rows = model::get_table_rows(&dir_entries, &options);
            let widths = &[
                Constraint::Length(options.left_column_witdh),
                Constraint::Length(options.right_column_width),
            ];
            let list = Table::new(rows)
                .block(Block::default().borders(Borders::ALL))
                .widths(widths)
                .highlight_style(style_selection);
            f.render_stateful_widget(list, size, &mut state);
        })?;

        // -- input
        use crossterm::event::KeyCode;
        match crossterm::event::read()? {
            crossterm::event::Event::Key(e) => match e.code {
                KeyCode::Esc | KeyCode::Char('q') => break,
                KeyCode::Char('j') => selected_index += 1,
                KeyCode::Char('k') => {
                    if selected_index > 0 {
                        selected_index -= 1
                    }
                }
                // KeyCode::Char('l') => {
                //     if let Some(index) = state.selected() {
                //         let name = descs[index].name;
                //     }
                // }
                KeyCode::Char('h') => {
                    options.show_hidden ^= true;
                    dir_entries = get_dir_entries(&cwd);
                }
                _ => {}
            },
            _ => {}
        }
    }

    terminal.show_cursor()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen // DisableMouseCapture
    )?;
    crossterm::terminal::disable_raw_mode()?;

    Ok(())
}
