static VARIABLES: phf::Map<&'static str, &'static str> = phf::phf_map! {
    "rsfm.show_hidden" => "boolean",
    "rsfm.entry_format" => "table",
    "rsfm.entry_format._" => "string", // array, matches 1, 2, 3...
};

const MAX_SIMILARITY_DISTANCE: usize = 3;

#[derive(Debug)]
struct VarDesc {
    hierarchy_name: String,
    type_name: &'static str,
}

fn parse_tree(path: &str, value: rlua::Value) -> Vec<VarDesc> {
    let mut actual_vars: Vec<VarDesc> = Vec::new();
    parse_tree_impl(path, value, &mut actual_vars);
    actual_vars
}

fn parse_tree_impl(path: &str, value: rlua::Value, vector: &mut Vec<VarDesc>) {
    if let rlua::Value::Table(table) = value {
        for pair in table.pairs::<String, rlua::Value>().into_iter() {
            match pair {
                Ok((key, value)) => {
                    let path = format!("{}.{}", path, key);
                    vector.push(VarDesc {
                        hierarchy_name: path.clone(),
                        type_name: value.type_name(),
                    });
                    parse_tree_impl(&path, value, vector);
                }
                Err(error) => {
                    eprintln!("error parsing config.lua: {}", error);
                    continue;
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

pub struct SyntaxError {
    errors: Vec<String>,
}

impl SyntaxError {
    fn new() -> SyntaxError {
        SyntaxError { errors: Vec::new() }
    }

    pub fn errors(self) -> Vec<String> {
        self.errors
    }

    pub fn has_error(&self) -> bool {
        !self.errors.is_empty()
    }
}

fn matches_array(var: &VarDesc) -> bool {
    let (table, inner) = match var.hierarchy_name.rsplit_once('.') {
        Some(tuple) => tuple,
        None => return false,
    };

    let table_type = VARIABLES.get(&table);

    if table_type.is_none() || (*table_type.unwrap()).ne("table") || inner.parse::<u16>().is_err() {
        false
    } else {
        match VARIABLES.get(&(table.to_owned() + "._")) {
            Some(inner_type) => (*inner_type).eq(var.type_name),
            None => false,
        }
    }
}

pub fn check(root_key: &str, root_value: rlua::Value) -> Result<(), SyntaxError> {
    let actual_vars = parse_tree(root_key, root_value);

    let mut error = SyntaxError::new();

    actual_vars
        .into_iter()
        .for_each(|var| match VARIABLES.get(&var.hierarchy_name) {
            Some(expected_type_name) => {
                if var.type_name.ne(*expected_type_name) {
                    let what = format!(
                        "Unexpected type '{}' for variable '{}', use '{}'",
                        var.type_name, var.hierarchy_name, expected_type_name
                    );
                    error.errors.push(what);
                }
            }
            None => {
                if matches_array(&var) {
                    return;
                }

                let what = match find_similar(
                    &var.hierarchy_name,
                    VARIABLES.keys(),
                    MAX_SIMILARITY_DISTANCE,
                ) {
                    Some(similar) => {
                        format!(
                            "Unknown variable '{}'. Did you mean '{}'?",
                            var.hierarchy_name, similar
                        )
                    }
                    None => {
                        format!("Unknown variable '{}'", var.hierarchy_name)
                    }
                };

                error.errors.push(what);
            }
        });

    if error.has_error() {
        Err(error)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn find_similar() {}
}
