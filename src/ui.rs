use tui::style::*;
use tui::widgets::*;

use std::io;

pub struct ViewOptions {
    pub show_hidden: bool,
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum EntryType {
    Dir,
    File,
    Link,
    Unknown,
}

impl EntryType {
    fn new(metadata: &std::fs::Metadata) -> EntryType {
        if metadata.is_dir() {
            EntryType::Dir
        } else if metadata.is_file() {
            EntryType::File
        } else if metadata.is_symlink() {
            EntryType::Link
        } else {
            EntryType::Unknown
        }
    }

    // fn priority(&self) -> u8 {
    //     *FILE_PRIORITY.get(self).unwrap()
    // }
}

use std::collections::HashMap;

lazy_static::lazy_static! {
    static ref FILE_STYLES: HashMap<EntryType, Style> = {
        let mut map = HashMap::new();
        map.insert(EntryType::Dir, Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));
        map.insert(EntryType::File, Style::default());
        map.insert(EntryType::Link, Style::default().fg(Color::Green).add_modifier(Modifier::ITALIC));
        map.insert(EntryType::Unknown, Style::default());
        map
    };
}

lazy_static::lazy_static! {
    static ref FILE_PRIORITY: HashMap<EntryType, u8> = {
        let mut map = HashMap::new();
        map.insert(EntryType::File, 2);
        map.insert(EntryType::Dir, 1);
        map.insert(EntryType::Link, 2);
        map.insert(EntryType::Unknown, 0);
        map
    };
}

fn get_info_string(metadata: &std::fs::Metadata) -> String {
    "".to_string()
}

fn parse_metadata(metadata: &Result<std::fs::Metadata, io::Error>) -> (String, Style) {
    match metadata {
        Ok(metadata) => {
            let entry_type = EntryType::new(metadata);
            (
                get_info_string(metadata),
                *FILE_STYLES.get(&entry_type).unwrap(),
            )
        }
        Err(error) => {
            eprintln!("{error}");
            (
                "<ERROR>".to_string(),
                *FILE_STYLES.get(&EntryType::Unknown).unwrap(),
            )
        }
    }
}

pub fn get_table_rows<'a>(entries: &'a Vec<std::fs::DirEntry>, opt: &ViewOptions) -> Vec<Row<'a>> {
    // struct Item {
    //     name: String,
    //     info: String,
    //     style: Style,
    // }

    entries
        .iter()
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();

            if opt.show_hidden == true {
                if let Some(c) = name.chars().nth(0) {
                    if c == '.' {
                        return None;
                    }
                }
            }

            let (info, style) = parse_metadata(&entry.metadata());
            // Some(Item { name, info, style })
            Some(Row::new(vec![name, info]).style(style))
        })
        .collect()
}
