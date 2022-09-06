static VARIABLES: phf::Map<&'static str, &'static str> = phf::phf_map! {
    "rsfm.show_hidden" => "boolean",
    "rsfm.column_left" => "table",
    "rsfm.column_left.width" => "integer",
    "rsfm.column_right" => "table",
    "rsfm.column_right.width" => "integer",
};

const MAX_DISTANCE: usize = 3;

struct VarDesc<'a> {
    hierarchy_name: String,
    type_name: &'a str,
}

fn parse_var_tree(path: &str, value: rlua::Value, vector: &mut Vec<VarDesc>) {
    if let rlua::Value::Table(table) = value {
        for pair in table.pairs::<String, rlua::Value>().into_iter() {
            match pair {
                Ok((key, value)) => {
                    let path = format!("{}.{}", path, key);
                    vector.push(VarDesc {
                        hierarchy_name: path.clone(),
                        type_name: value.type_name(),
                    });
                    parse_var_tree(&path, value, vector);
                }
                Err(error) => {
                    eprintln!("error parsing config.lua: {}", error);
                    continue;
                }
            }
        }
    }
}

fn find_similar_var(target_name: &str, max_distance: usize) -> Option<&str> {
    let mut distance = max_distance + 1;
    let mut similar = "";

    for expected_name in VARIABLES.keys() {
        let current_distance = levenshtein::levenshtein(&expected_name, target_name);
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

pub fn check(root_key: &str, root_value: rlua::Value) -> Result<(), SyntaxError> {
    let mut actual_vars: Vec<VarDesc> = Vec::new();
    parse_var_tree(root_key, root_value, &mut actual_vars);

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
                let what = match find_similar_var(&var.hierarchy_name, MAX_DISTANCE) {
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
mod tests {}
