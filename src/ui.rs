use tui::style::*;
use tui::widgets::*;

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

pub struct ViewOptions {
    pub show_hidden: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
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
}

fn get_info_string(metadata: &std::fs::Metadata) -> String {
    "".to_string()
}

fn parse_metadata(metadata: &Option<std::fs::Metadata>) -> (String, Style) {
    match metadata {
        Some(metadata) => {
            let entry_type = EntryType::new(metadata);
            (
                get_info_string(metadata),
                *FILE_STYLES.get(&entry_type).unwrap(),
            )
        }
        None => (
            "<???>".to_string(),
            *FILE_STYLES.get(&EntryType::Unknown).unwrap(),
        ),
    }
}

#[derive(Debug, Clone)]
struct Item {
    name: String,
    entry_type: EntryType,
    metadata: Option<std::fs::Metadata>,
}

struct Part {
    begin: usize,
    end: usize,
}

fn split_into_parts(items: &Vec<Item>) -> Vec<Part> {
    if items.is_empty() {
        return Vec::new();
    }

    let mut result: Vec<Part> = Vec::new();

    let mut current_type = items.first().unwrap().entry_type;
    let mut current_priority = *FILE_PRIORITY
        .get(&items.first().unwrap().entry_type)
        .unwrap();
    let mut begin = 0usize;

    for (i, item) in items.iter().enumerate() {
        if current_type != item.entry_type {
            current_type = item.entry_type;
            if current_priority != *FILE_PRIORITY.get(&item.entry_type).unwrap() {
                result.push(Part { begin, end: i });
                begin = i;
                current_priority = *FILE_PRIORITY.get(&item.entry_type).unwrap();
            }
        }
    }
    result.push(Part {
        begin,
        end: items.len(),
    });
    result
}

pub fn get_table_rows<'a>(entries: &'a Vec<std::fs::DirEntry>, opt: &ViewOptions) -> Vec<Row<'a>> {
    let items: Vec<_> = entries
        .iter()
        .map(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            let metadata = match entry.metadata() {
                Ok(metadata) => Some(metadata),
                Err(error) => {
                    eprintln!("{error}");
                    None
                }
            };

            let entry_type = match metadata {
                Some(ref metadata) => EntryType::new(&metadata),
                None => EntryType::Unknown,
            };

            Item {
                name,
                entry_type,
                metadata,
            }
        })
        .collect();

    let mut items: Vec<_> = match opt.show_hidden {
        false => items
            .into_iter()
            .filter(|item| match item.name.chars().nth(0) {
                Some(c) if c == '.' => false,
                _ => true,
            })
            .collect(),
        true => items,
    };

    items.sort_by(|a, b| {
        let get_priority = |item: &Item| -> u8 { *FILE_PRIORITY.get(&item.entry_type).unwrap() };
        get_priority(&a)
            .partial_cmp(&get_priority(&b))
            .unwrap_or(std::cmp::Ordering::Less)
    });

    let items_parts = split_into_parts(&items);

    items_parts.into_iter().for_each(|part| {
        let slice = &mut items[part.begin..part.end];
        slice.sort_by(|a, b| {
            let a_char = a.name.chars().nth(0).unwrap_or(' ');
            let b_char = b.name.chars().nth(0).unwrap_or(' ');
            a_char
                .partial_cmp(&b_char)
                .unwrap_or(std::cmp::Ordering::Less)
        });
    });

    items
        .into_iter()
        .map(|item| {
            let (info, style) = parse_metadata(&item.metadata);
            Row::new(vec![item.name, info]).style(style)
        })
        .collect()
}

#[cfg(test)]
mod tests {

    macro_rules! item_vec {
    ( $( $x:expr ),* ) => {
        {
            let mut temp = Vec::new();
            $(
                // temp.push($x);
                temp.push(Item{
                    name: String::new(),
                    entry_type: $x,
                    metadata: None,
                });
            )*
            temp
        }
    };
}

    #[test]
    fn as_slice_by_type() {
        use super::*;

        let items = item_vec!(
            EntryType::Unknown,
            EntryType::Unknown,
            EntryType::Unknown,
            EntryType::Dir,
            EntryType::Dir,
            EntryType::File,
            EntryType::File,
            EntryType::Link
        );

        let slices = split_into_parts(&items);
        assert_eq!(slices.len(), 3);

        assert_eq!(slices[0].begin, 0);
        assert_eq!(slices[0].end, 3);

        assert_eq!(slices[1].begin, 3);
        assert_eq!(slices[1].end, 5);

        assert_eq!(slices[2].begin, 5);
        assert_eq!(slices[2].end, 8);
    }
}
