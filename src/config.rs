pub mod parser;

pub mod column {
    #[derive(Debug)]
    pub enum ColumnType {
        Name,
        Size,
    }

    impl ColumnType {
        fn from(typename: &str) -> Result<Self, ()> {
            match typename {
                "name" => Ok(ColumnType::Name),
                "size" => Ok(ColumnType::Size),
                _ => Err(()),
            }
        }
    }

    #[derive(Debug)]
    pub struct Column {
        pub column_type: ColumnType,
        pub width: u16,
        pub is_fixed_width: bool,
    }

    impl Column {
        pub fn new(column_typename: &str, width: u16, is_fixed_width: bool) -> Self {
            Column {
                column_type: ColumnType::from(column_typename)
                    .unwrap_or_else(|()| panic!("Unknown column type: {column_typename}")),
                width,
                is_fixed_width,
            }
        }
    }
}

#[derive(Debug)]
pub struct ViewOptions {
    pub show_hidden: bool,
    pub columns: Vec<column::Column>,
    pub column_left_width: u16,
    pub column_right_width: u16,
}

impl ViewOptions {
    pub fn default() -> ViewOptions {
        ViewOptions {
            show_hidden: false,
            columns: vec![column::Column::new("name", 1, false)],
            column_left_width: 50,
            column_right_width: 10,
        }
    }
}

pub fn read_config(path: &std::path::Path) -> ViewOptions {
    let mut result = ViewOptions::default();

    match std::fs::read_to_string(path) {
        Ok(config_source) => {
            let lua = rlua::Lua::new();

            match lua.context(|ctx| -> Result<(), rlua::Error> {
                let rsfm = ctx.create_table()?;
                let globals = ctx.globals();

                globals.set("rsfm", rsfm)?;

                ctx.load(&config_source).set_name("config")?.exec()?;

                let rsfm = globals.get::<_, rlua::Value>("rsfm")?;

                if let Err(errors) = parser::parse_syntax("rsfm", rsfm.clone()) {
                    eprintln!("Configuration syntax error: ");
                    for e in errors {
                        eprintln!("{e}");
                    }
                };

                result = parser::parse_values(rsfm)?;

                Ok(())
            }) {
                Ok(()) => result,
                Err(error) => {
                    eprintln!("Lua execution error: {error}");
                    result
                }
            }
        }
        Err(error) => {
            eprintln!("Error reading configuration file: {error}");
            result
        }
    }
}
