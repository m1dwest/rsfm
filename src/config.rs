pub mod parser;

pub mod column {
    #[derive(Debug, PartialEq)]
    pub enum ColumnType {
        Name,
        Size,
    }

    impl ColumnType {
        pub fn from(typename: &str) -> Result<Self, ()> {
            match typename.to_lowercase().as_str() {
                "name" => Ok(ColumnType::Name),
                "size" => Ok(ColumnType::Size),
                _ => Err(()),
            }
        }
    }

    #[derive(Debug, PartialEq)]
    pub enum Alignment {
        Left,
        Center,
        Right,
    }

    impl Alignment {
        pub fn from(typename: &str) -> Result<Self, ()> {
            match typename.to_lowercase().as_str() {
                "left" => Ok(Alignment::Left),
                "center" => Ok(Alignment::Center),
                "right" => Ok(Alignment::Right),
                _ => Err(()),
            }
        }
    }

    #[derive(Debug, PartialEq)]
    pub struct Column {
        pub column_type: ColumnType,
        pub width: u16,
        pub is_fixed_width: bool,
        pub alignment: Alignment,
    }

    impl Column {
        pub fn new(column_typename: &str, width: u16, is_fixed_width: bool) -> Self {
            Column {
                column_type: ColumnType::from(column_typename)
                    .unwrap_or_else(|()| panic!("Unknown column type: {column_typename}")),
                width,
                is_fixed_width,
                alignment: Alignment::Left,
            }
        }
    }
}

#[derive(Debug)]
pub struct ViewOptions {
    pub show_hidden: bool,
    pub entry_format: Vec<column::Column>,
}

impl ViewOptions {
    pub fn default() -> ViewOptions {
        ViewOptions {
            show_hidden: false,
            entry_format: Vec::new(),
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
