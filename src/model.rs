use tui::layout::Constraint;
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

pub struct TableData<'a> {
    pub rows: Vec<Row<'a>>,
    pub widths: Vec<Constraint>,
}

pub fn get_table_data<'a>(
    entries: &'a Vec<std::fs::DirEntry>,
    options: &config::ViewOptions,
    terminal_width: u16,
) -> TableData<'a> {
    let items: Vec<_> = entries.iter().map(Item::from).collect();

    let mut items: Vec<_> = items
        .into_iter()
        .filter(|item| match item.name.chars().nth(0) {
            Some(c) if !options.show_hidden && c == '.' => false,
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

    let widths = generate_widths(&options, terminal_width);

    let rows = items
        .into_iter()
        .map(|item| {
            let style = *ITEM_STYLES.get(&item.entry_type).unwrap();
            Row::new(generate_columns(item, &options.entry_format, &widths)).style(style)
        })
        .collect();

    TableData {
        rows,
        widths: widths
            .into_iter()
            .map(|width| Constraint::Length(width))
            .collect(),
    }
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

fn generate_widths(options: &config::ViewOptions, total_width: u16) -> Vec<u16> {
    const BORDER_WIDTH: u16 = 1;

    let terminal_width = {
        let column_count = options.entry_format.len() as u16;
        let occupied_width = 2 * BORDER_WIDTH
            + if column_count > 1 {
                column_count - 1
            } else {
                0
            };
        if total_width < occupied_width {
            0
        } else {
            total_width - occupied_width
        }
    };

    let mut sum_relative = 0u16;
    let mut sum_fixed = 0u16;
    for column in &options.entry_format {
        if column.is_fixed_width {
            sum_fixed += column.width;
        } else {
            sum_relative += column.width;
        };
    }

    let width_unit = if sum_relative == 0 || sum_fixed >= terminal_width {
        0.0
    } else {
        (terminal_width - sum_fixed) as f64 / sum_relative as f64
    };

    options
        .entry_format
        .iter()
        .map(|column| {
            if column.is_fixed_width {
                column.width
            } else {
                (column.width as f64 * width_unit) as u16
            }
        })
        .collect()
}

fn generate_columns(
    item: Item,
    columns: &Vec<config::column::Column>,
    widths: &Vec<u16>,
) -> Vec<String> {
    use config::column;
    use config::column::ColumnType;

    use pad::PadStr;

    const PAD_CHAR: char = ' ';

    let to_pad_alignment = |alignment: &column::Alignment| match alignment {
        column::Alignment::Left => pad::Alignment::Left,
        column::Alignment::Center => pad::Alignment::Middle,
        column::Alignment::Right => pad::Alignment::Right,
    };

    columns
        .iter()
        .zip(widths)
        .map(|(column, width)| {
            let alignment = to_pad_alignment(&column.alignment);

            match column.column_type {
                ColumnType::Name => {
                    generate_name(&item).pad(*width as usize, PAD_CHAR, alignment, true)
                }
                ColumnType::Size => {
                    generate_size(&item).pad(*width as usize, PAD_CHAR, alignment, true)
                }
            }
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
