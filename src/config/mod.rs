use crate::model;
mod syntax;

pub fn read_config(path: &std::path::Path) -> model::ViewOptions {
    let mut result_config = model::ViewOptions::default();

    let lua = rlua::Lua::new();

    if let Ok(code) = std::fs::read_to_string("config.lua") {
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

            match rsfm {
                rlua::Value::Table(rsfm) => {
                    if let Ok(show_hidden) = rsfm.get::<_, bool>("show_hidden") {
                        println!("show_hidden: {show_hidden}");
                    }

                    if let Ok(column_left) = rsfm.get::<_, rlua::Table>("column_left") {
                        if let Ok(width) = column_left.get::<_, rlua::Integer>("width") {
                            println!("column_left.width: {width}");
                        }
                    }

                    if let Ok(column_right) = rsfm.get::<_, rlua::Table>("column_right") {
                        if let Ok(width) = column_right.get::<_, rlua::Integer>("width") {
                            println!("column_right.width: {width}");
                        }
                    }
                }
                _ => {
                    return Err(rlua::Error::RuntimeError(
                        "Cannot read 'rsfm' as table".to_string(),
                    ));
                }
            }

            Ok::<(), rlua::Error>(())
        }) {
            Ok(()) => {
                // OK
                // TODO customize
                return result_config;
            }
            Err(error) => {
                println!("lue execution error: {error}");
            }
        }
    };

    // TODO error message
    model::ViewOptions::default()
}
