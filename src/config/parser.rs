use super::{column, ViewOptions};

static VARIABLES: phf::Map<&'static str, &'static str> = phf::phf_map! {
    "rsfm.show_hidden" => "boolean",
    "rsfm.entry_format" => "table",
    "rsfm.entry_format.{}" => "table",
    "rsfm.entry_format.{}.type" => "string",
    "rsfm.entry_format.{}.width" => "integer",
    "rsfm.entry_format.{}.is_fixed_width" => "boolean",
};

const MAX_SIMILARITY_DISTANCE: usize = 3;

#[derive(Debug)]
pub struct VarDesc {
    name: String,
    type_name: &'static str,
}

fn parse_tree(name: &str, value: rlua::Value) -> Vec<VarDesc> {
    let mut actual_vars: Vec<VarDesc> = Vec::new();
    parse_tree_impl(name, value, &mut actual_vars);
    actual_vars
}

fn parse_tree_impl(name: &str, value: rlua::Value, vector: &mut Vec<VarDesc>) {
    if let rlua::Value::Table(table) = value {
        for pair in table.pairs::<String, rlua::Value>().into_iter() {
            match pair {
                Ok((key, value)) => {
                    let name = format!("{}.{}", name, key);
                    vector.push(VarDesc {
                        name: name.clone(),
                        type_name: value.type_name(),
                    });
                    parse_tree_impl(&name, value, vector);
                }
                Err(error) => {
                    eprintln!("error parsing config.lua: {}", error);
                }
            }
        }
    }
}

fn find_similar<'a, T>(target: &'a str, src_list: T, max_distance: usize) -> Option<&str>
where
    T: Iterator<Item = &'a &'a str>,
{
    let mut distance = max_distance + 1;
    let mut similar = "";

    for expected_name in src_list {
        let current_distance = levenshtein::levenshtein(&expected_name, target);
        if current_distance < distance {
            distance = current_distance;
            similar = expected_name;
        }
    }

    return if distance <= max_distance {
        Some(similar)
    } else {
        None
    };
}

// fn matches_array(var: &VarDesc) -> bool {
//     let (table, inner) = match var.name.rsplit_once('.') {
//         Some(tuple) => tuple,
//         None => return false,
//     };

//     let table_type = VARIABLES.get(&table);

//     if table_type.is_none() || (*table_type.unwrap()).ne("table") || inner.parse::<u16>().is_err() {
//         false
//     } else {
//         match VARIABLES.get(&(table.to_owned() + "{}")) {
//             Some(inner_type) => (*inner_type).eq(var.type_name),
//             None => false,
//         }
//     }
// }

fn replace_array_index(var: &str) -> String {
    use itertools::Itertools;

    const SEPARATOR: &str = ".";
    var.split(SEPARATOR)
        .map(|var| {
            if var.parse::<u16>().is_ok() {
                "{}"
            } else {
                var
            }
        })
        .join(SEPARATOR)
}

pub type CheckResult = Result<Vec<VarDesc>, Vec<String>>;

pub fn parse_syntax(root_key: &str, root_value: rlua::Value) -> CheckResult {
    let actual_vars = parse_tree(root_key, root_value);

    let mut errors = Vec::new();
    let mut names_to_skip: Vec<&str> = Vec::new();

    actual_vars.iter().for_each(|var| {
        // skip checking underlying table entries if 'Table' is unexpected type
        if names_to_skip
            .iter()
            .find(|&&x| var.name.starts_with(x))
            .is_some()
        {
            return;
        }

        let no_arr_index = replace_array_index(&var.name);
        match VARIABLES.get(&no_arr_index) {
            Some(expected_type_name) => {
                if var.type_name.ne(*expected_type_name) {
                    let what = format!(
                        "Unexpected type '{}' for variable '{}', use '{}'",
                        var.type_name, var.name, expected_type_name
                    );
                    errors.push(what);
                    names_to_skip.push(&var.name);
                }
            }
            None => {
                let what =
                    match find_similar(&no_arr_index, VARIABLES.keys(), MAX_SIMILARITY_DISTANCE) {
                        Some(similar) => {
                            format!(
                                "Unknown variable '{}'. Did you mean '{}'?",
                                no_arr_index, similar
                            )
                        }
                        None => {
                            format!("Unknown variable '{}'", no_arr_index)
                        }
                    };

                errors.push(what);
            }
        }
    });

    if errors.is_empty() {
        Ok(actual_vars)
    } else {
        Err(errors)
    }
}

fn parse_show_hidden(table: &rlua::Table, default: bool) -> bool {
    if let Ok(value) = table.get::<_, bool>("show_hidden") {
        value
    } else {
        default
    }
}

fn parse_entry_format(table: &rlua::Table) -> Vec<column::Column> {
    if let Ok(entry_format) = table.get::<_, rlua::Table>("entry_format") {
        entry_format
            .pairs()
            .filter_map(
                |result: Result<(u16, rlua::Table), rlua::Error>| match result {
                    Ok(pair) => {
                        let (index, column_table) = pair;

                        let typename = match column_table.get::<_, String>("type") {
                            Ok(typename) => typename,
                            Err(error) => {
                                eprintln!(
                                    "Error parsing 'rsfm.entry_format.{index}.type': {error}"
                                );
                                return None;
                            }
                        };

                        let column_type = match column::ColumnType::from(&typename) {
                            Ok(column_type) => column_type,
                            Err(()) => {
                                eprintln!("Unknown column type name: {typename}");
                                return None;
                            }
                        };

                        let width = match column_table.get::<_, u16>("width") {
                            Ok(width) => width,
                            Err(error) => {
                                eprintln!(
                                    "Error parsing 'rsfm.entry_format.{index}.width': {error}"
                                );
                                return None;
                            }
                        };

                        if width == 0 {
                            eprintln!(
                                "Error parsing 'rsfm.entry_format.{index}.width': value can not be 0"
                            );
                            return None;
                        }

                        let is_fixed_width = match column_table.contains_key("is_fixed_width") {
                            Ok(contains) if contains != false => {
                                column_table.get::<_, bool>("is_fixed_width").unwrap()
                            },
                            _ => {
                                eprintln!(
                                    "Error parsing 'rsfm.entry_format.{index}.is_fixed_width': value not found"
                                );
                                return None;
                            }
                        };

                        Some(column::Column { column_type, width, is_fixed_width})
                    }
                    Err(error) => {
                        eprintln!("Error parsing 'rsfm.entry_format': {error}");
                        None
                    }
                },
            )
            .collect()
    } else {
        Vec::new()
    }
}

fn parse_root(table: rlua::Table) -> ViewOptions {
    let mut options = ViewOptions::default();

    options.show_hidden = parse_show_hidden(&table, options.show_hidden);
    options.entry_format = parse_entry_format(&table);

    options
}

pub fn parse_values(root: rlua::Value) -> Result<ViewOptions, rlua::Error> {
    match root {
        rlua::Value::Table(root) => {
            let opt = parse_root(root);
            Ok(opt)
        }
        _ => Err(rlua::Error::RuntimeError(
            "Cannot read config.lua root table".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn replace_array_index() {
        let actual = super::replace_array_index("root.var1.1.var2.2");
        assert_eq!(actual, "root.var1.{}.var2.{}");
    }
}
