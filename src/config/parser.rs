use super::ViewOptions;

static VARIABLES: phf::Map<&'static str, &'static str> = phf::phf_map! {
    "rsfm.show_hidden" => "boolean",
    "rsfm.entry_format" => "table",
    "rsfm.entry_format{}" => "string", // array, matches 1, 2, 3...
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

fn matches_array(var: &VarDesc) -> bool {
    let (table, inner) = match var.name.rsplit_once('.') {
        Some(tuple) => tuple,
        None => return false,
    };

    let table_type = VARIABLES.get(&table);

    if table_type.is_none() || (*table_type.unwrap()).ne("table") || inner.parse::<u16>().is_err() {
        false
    } else {
        match VARIABLES.get(&(table.to_owned() + "{}")) {
            Some(inner_type) => (*inner_type).eq(var.type_name),
            None => false,
        }
    }
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

        match VARIABLES.get(&var.name) {
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
                if matches_array(&var) {
                    return;
                }

                let what = match find_similar(&var.name, VARIABLES.keys(), MAX_SIMILARITY_DISTANCE)
                {
                    Some(similar) => {
                        format!(
                            "Unknown variable '{}'. Did you mean '{}'?",
                            var.name, similar
                        )
                    }
                    None => {
                        format!("Unknown variable '{}'", var.name)
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

fn parse_root(table: rlua::Table) -> ViewOptions {
    let mut options = ViewOptions::default();

    if let Ok(show_hidden) = table.get::<_, bool>("show_hidden") {
        options.show_hidden = show_hidden;
    }

    if let Ok(entry_format) = table.get::<_, rlua::Table>("entry_format") {
        let re = regex::Regex::new(r"^(?P<name>\w+)(?::(?P<size>\d+)(?P<is_fixed>\w)?)?$").unwrap();

        options.columns = entry_format
            .pairs()
            .filter_map(|result: Result<(u16, String), rlua::Error>| match result {
                Ok(pair) => {
                    let string = pair.1;
                    if re.is_match(&string) {
                        // TODO check for syntax errors
                        let capture = re.captures(&string)?;
                        let name = capture.name("name").unwrap().as_str();
                        let size = capture
                            .name("size")
                            .map_or(1_u16, |m| m.as_str().parse::<u16>().unwrap());
                        let is_fixed = capture
                            .name("is_fixed")
                            .map_or(false, |m| m.as_str().eq("f"));
                        Some(super::column::Column::new(name, size, is_fixed))
                    } else {
                        eprintln!("Error parsing 'rsfm.entry_format' value '{string}'");
                        None
                    }
                }
                Err(error) => {
                    eprintln!("Error parsing 'rsfm.entry_format': {error}");
                    None
                }
            })
            .collect();
    }

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
    fn find_similar() {}
}
