mod syntax;

#[derive(Debug)]
pub enum MetadataFormat {
    Fixed(u16),
    Relative(u16),
}

#[derive(Debug)]
pub enum EntryMetadata {
    Name(MetadataFormat),
    Info(MetadataFormat),
}

impl EntryMetadata {
    fn new(name: &str, size: u16, is_fixed: bool) -> Result<EntryMetadata, ()> {
        let format = if is_fixed {
            MetadataFormat::Fixed(size)
        } else {
            MetadataFormat::Relative(size)
        };

        match name {
            "name" => Ok(EntryMetadata::Name(format)),
            "info" => Ok(EntryMetadata::Info(format)),
            _ => Err(()),
        }
    }
}

pub struct ViewOptions {
    pub show_hidden: bool,
    pub entry_format: Vec<EntryMetadata>,
    pub column_left_width: u16,
    pub column_right_width: u16,
}

impl ViewOptions {
    pub fn default() -> ViewOptions {
        ViewOptions {
            show_hidden: false,
            entry_format: vec![EntryMetadata::Name(MetadataFormat::Relative(1))],
            column_left_width: 50,
            column_right_width: 10,
        }
    }
}

fn parse_variables(root: rlua::Value) -> Result<ViewOptions, rlua::Error> {
    match root {
        rlua::Value::Table(root) => {
            let mut options = ViewOptions::default();

            if let Ok(show_hidden) = root.get::<_, bool>("show_hidden") {
                options.show_hidden = show_hidden;
            }

            if let Ok(entry_format) = root.get::<_, rlua::Table>("entry_format") {
                let re = regex::Regex::new(r"^(?P<name>\w+)(?::(?P<size>\d+)(?P<is_fixed>\w)?)?$")
                    .unwrap();

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

            if let Ok(column_left) = root.get::<_, rlua::Table>("column_left") {
                if let Ok(width) = column_left.get::<_, rlua::Integer>("width") {
                    match u16::try_from(width) {
                        Ok(width) => options.column_left_width = width,
                        Err(_) => {
                            return Err(rlua::Error::FromLuaConversionError {
                                from: "integer",
                                to: "u8",
                                message: Some(
                                    "Conversion failed for 'column_left.width'".to_string(),
                                ),
                            });
                        }
                    }
                }
            }

            if let Ok(column_right) = root.get::<_, rlua::Table>("column_right") {
                if let Ok(width) = column_right.get::<_, rlua::Integer>("width") {
                    match u16::try_from(width) {
                        Ok(width) => options.column_right_width = width,
                        Err(_) => {
                            return Err(rlua::Error::FromLuaConversionError {
                                from: "integer",
                                to: "u8",
                                message: Some(
                                    "Conversion failed for 'column_right.width'".to_string(),
                                ),
                            });
                        }
                    }
                }
            }

            Ok(options)
        }
        _ => Err(rlua::Error::RuntimeError(
            "Cannot read config.lua root table".to_string(),
        )),
    }
}

pub fn read_config(path: &std::path::Path) -> ViewOptions {
    let mut result_config = ViewOptions::default();

    let lua = rlua::Lua::new();

    match std::fs::read_to_string(path) {
        Ok(code) => {
            match lua.context(|ctx| {
                let rsfm = ctx.create_table()?;
                let globals = ctx.globals();

                globals.set("rsfm", rsfm)?;
                ctx.load(&code).set_name("config")?.exec()?;

                let rsfm = globals.get::<_, rlua::Value>("rsfm")?;

                if let Err(error) = syntax::check("rsfm", rsfm.clone()) {
                    eprintln!("Configuration syntax error: ");
                    for e in error.errors() {
                        eprintln!("{e}");
                    }
                };

                result_config = parse_variables(rsfm)?;

                Ok::<(), rlua::Error>(())
            }) {
                Ok(()) => result_config,
                Err(error) => {
                    eprintln!("Lua execution error: {error}");
                    result_config
                }
            }
        }
        Err(error) => {
            eprintln!("IO error: {error}");
            result_config
        }
    }
}
