use super::{EntryMetadata, ViewOptions};

fn parse_root(table: rlua::Table) -> ViewOptions {
    let mut options = ViewOptions::default();

    if let Ok(show_hidden) = table.get::<_, bool>("show_hidden") {
        options.show_hidden = show_hidden;
    }

    if let Ok(entry_format) = table.get::<_, rlua::Table>("entry_format") {
        let re = regex::Regex::new(r"^(?P<name>\w+)(?::(?P<size>\d+)(?P<is_fixed>\w)?)?$").unwrap();

        options.entry_format = entry_format
            .pairs()
            .filter_map(|result: Result<(u16, String), rlua::Error>| match result {
                Ok(pair) => {
                    let string = pair.1;
                    if re.is_match(&string) {
                        let capture = re.captures(&string)?;
                        let name = capture.name("name").unwrap().as_str();
                        let size = capture
                            .name("size")
                            .map_or(1_u16, |m| m.as_str().parse::<u16>().unwrap());
                        let is_fixed = capture
                            .name("is_fixed")
                            .map_or(false, |m| m.as_str().eq("f"));
                        EntryMetadata::new(name, size, is_fixed).ok()
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

pub fn parse(root: rlua::Value) -> Result<ViewOptions, rlua::Error> {
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
