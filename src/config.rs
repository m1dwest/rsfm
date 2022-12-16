mod parser;
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

                result_config = parser::parse(rsfm)?;

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
