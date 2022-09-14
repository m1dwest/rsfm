use crate::model;
mod syntax;

fn parse_variables(root: rlua::Value) -> Result<model::ViewOptions, rlua::Error> {
    match root {
        rlua::Value::Table(root) => {
            let mut options = model::ViewOptions::default();

            if let Ok(show_hidden) = root.get::<_, bool>("show_hidden") {
                options.show_hidden = show_hidden;
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

pub fn read_config(path: &std::path::Path) -> model::ViewOptions {
    let mut result_config = model::ViewOptions::default();

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
