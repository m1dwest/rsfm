use tui::style::*;
use tui::widgets::*;

use std::collections::HashMap;

use crate::config;
mod details;

lazy_static::lazy_static! {
    static ref ITEM_STYLES: HashMap<EntryType, Style> = {
        let mut map = HashMap::new();
        map.insert(EntryType::Dir, Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));
        map.insert(EntryType::File, Style::default());
        map.insert(EntryType::Link, Style::default().fg(Color::Green).add_modifier(Modifier::ITALIC));
        map.insert(EntryType::Unknown, Style::default());
        map
    };
}

lazy_static::lazy_static! {
    static ref ITEM_PRIORITY: HashMap<EntryType, u8> = {
        let mut map = HashMap::new();
        map.insert(EntryType::File, 2);
        map.insert(EntryType::Dir, 1);
        map.insert(EntryType::Link, 2);
        map.insert(EntryType::Unknown, 0);
        map
    };
}

const DIR_SIZE_PLACEHOLDER: &str = "<DIR>";
const LINK_SIZE_PLACEHOLDER: &str = " --> ";
const UNKNOWN_SIZE_PLACEHOLDER: &str = "<???>";

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

#[derive(Debug, Clone)]
struct Item {
    name: String,
    entry_type: EntryType,
    metadata: Option<std::fs::Metadata>,
}

impl Item {
    fn from(entry: &std::fs::DirEntry) -> Self {
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
    }
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
    let mut current_priority = *ITEM_PRIORITY
        .get(&items.first().unwrap().entry_type)
        .unwrap();
    let mut begin = 0usize;

    for (i, item) in items.iter().enumerate() {
        if current_type != item.entry_type {
            current_type = item.entry_type;
            if current_priority != *ITEM_PRIORITY.get(&item.entry_type).unwrap() {
                result.push(Part { begin, end: i });
                begin = i;
                current_priority = *ITEM_PRIORITY.get(&item.entry_type).unwrap();
            }
        }
    }
    result.push(Part {
        begin,
        end: items.len(),
    });
    result
}

pub fn get_table_rows<'a>(
    entries: &'a Vec<std::fs::DirEntry>,
    opt: &config::ViewOptions,
) -> Vec<Row<'a>> {
    let items: Vec<_> = entries.iter().map(Item::from).collect();

    let mut items: Vec<_> = items
        .into_iter()
        .filter(|item| match item.name.chars().nth(0) {
            Some(c) if !opt.show_hidden && c == '.' => false,
            _ => true,
        })
        .collect();

    items.sort_by(|a, b| {
        let get_priority = |item: &Item| -> u8 { *ITEM_PRIORITY.get(&item.entry_type).unwrap() };
        get_priority(&a)
            .partial_cmp(&get_priority(&b))
            .unwrap_or(std::cmp::Ordering::Less)
    });

    split_into_parts(&items).into_iter().for_each(|part| {
        items[part.begin..part.end].sort_by(|a, b| {
            a.name
                .partial_cmp(&b.name)
                .unwrap_or(std::cmp::Ordering::Less)
        });
    });

    items
        .into_iter()
        .map(|item| {
            // let (info, style) = parse_metadata(&item.metadata, opt.column_right_width);
            let style = *ITEM_STYLES.get(&item.entry_type).unwrap();
            Row::new(generate_columns(item, &opt.entry_format)).style(style)
            // Row::new(vec![item.name, info]).style(style)
        })
        .collect()
}

fn generate_name(item: &Item) -> String {
    item.name.clone()
}

fn generate_size(item: &Item) -> String {
    match &item.metadata {
        Some(metadata) => {
            if metadata.is_dir() {
                String::from(DIR_SIZE_PLACEHOLDER)
            } else if metadata.is_file() {
                let (size, postfix) = details::human_readable_size(metadata.len());
                format!("{} {}", size, postfix)
            } else if metadata.is_symlink() {
                String::from(LINK_SIZE_PLACEHOLDER)
            } else {
                String::from(UNKNOWN_SIZE_PLACEHOLDER)
            }
        }
        None => String::from(UNKNOWN_SIZE_PLACEHOLDER),
    }
}

fn generate_columns(item: Item, columns: &Vec<config::column::Column>) -> Vec<String> {
    use config::column::ColumnType;

    columns
        .iter()
        .map(|column| match column.column_type {
            ColumnType::Name => generate_name(&item),
            ColumnType::Size => generate_size(&item),
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
