use tui::style::*;
use tui::widgets::*;

use std::{fs, io};

mod actions;

lazy_static::lazy_static! {
    static ref FILE_STYLES: std::collections::HashMap<FileType, Style> = {
        let mut map = std::collections::HashMap::new();
        map.insert(FileType::File, Style::default());
        map.insert(FileType::Dir, Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));
        map.insert(FileType::Link, Style::default().fg(Color::Green).add_modifier(Modifier::ITALIC));
        map
    };
}

lazy_static::lazy_static! {
    static ref FILE_PRIORITY: std::collections::HashMap<FileType, u8> = {
        let mut map = std::collections::HashMap::new();
        map.insert(FileType::Dir, 0);
        map.insert(FileType::File, 1);
        map.insert(FileType::Link, 1);
        map
    };
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum FileType {
    File,
    Dir,
    Link,
}

impl FileType {
    fn from(metadata: fs::Metadata) -> FileType {
        if metadata.is_dir() {
            FileType::Dir
        } else if metadata.is_symlink() {
            FileType::Link
        } else {
            FileType::File
        }
    }

    // fn priority(&self) -> u8 {
    //     *FILE_PRIORITY.get(self).unwrap()
    // }
}

struct FileInfo {
    name: String,
    r#type: FileType,
}

fn read_dir(path: &str, show_hidden: bool) -> Result<Vec<ListItem>, std::io::Error> {
    let dir = fs::read_dir(std::path::Path::new(path))?;

    let (mut dirs, mut files): (Vec<_>, Vec<_>) = dir
        .map(|dir| -> FileInfo {
            let dir = dir.unwrap();
            let name = dir.file_name().into_string().unwrap();
            let r#type = FileType::from(dir.metadata().unwrap());
            FileInfo { name, r#type }
        })
        .filter(|dir| show_hidden || dir.name.chars().nth(0).unwrap() != '.')
        .partition(|dir| dir.r#type == FileType::Dir);

    dirs.sort_by(|a, b| a.name.cmp(&b.name));
    files.sort_by(|a, b| a.name.cmp(&b.name));

    dirs.append(&mut files);

    let list = dirs
        .iter()
        .map(|info| {
            ListItem::new::<String>(info.name.clone())
                .style(*FILE_STYLES.get(&info.r#type).unwrap())
        })
        .collect();

    Ok(list)
}

fn main() -> Result<(), io::Error> {
    let stdout = io::stdout();
    let backend = tui::backend::CrosstermBackend::new(stdout);
    let mut terminal = tui::terminal::Terminal::new(backend)?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::EnterAlternateScreen
    )?;
    crossterm::terminal::enable_raw_mode()?;

    let mut selected_index = 0;
    let mut show_hidden = false;

    let mut file_list = read_dir("/home/midwest", show_hidden).unwrap();
    loop {
        // -- draw
        let style_selection = Style::default().fg(Color::Black).bg(Color::LightYellow);

        let mut state = ListState::default();
        state.select(Some(selected_index));

        terminal.draw(|f| {
            let size = f.size();
            let list = List::new(file_list.clone())
                .block(Block::default().borders(Borders::ALL))
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
                KeyCode::Char('h') => {
                    show_hidden ^= true;
                    file_list = read_dir("/home/midwest", show_hidden).unwrap();
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
