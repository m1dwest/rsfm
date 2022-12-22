use rsfm::column;
use rsfm::parser::CheckResult;
use rsfm::ViewOptions;

fn parse_syntax(source: &str) -> CheckResult {
    rlua::Lua::new().context(|ctx| -> CheckResult {
        let rsfm = ctx.create_table().unwrap();
        let globals = ctx.globals();

        globals.set("rsfm", rsfm).unwrap();

        ctx.load(source).exec().unwrap();

        let rsfm = globals.get::<_, rlua::Value>("rsfm").unwrap();
        rsfm::parser::parse_syntax("rsfm", rsfm)
    })
}

fn parse_values(source: &str) -> ViewOptions {
    rlua::Lua::new().context(|ctx| -> ViewOptions {
        let rsfm = ctx.create_table().unwrap();
        let globals = ctx.globals();

        globals.set("rsfm", rsfm).unwrap();

        ctx.load(source).exec().unwrap();

        let rsfm = globals.get::<_, rlua::Value>("rsfm").unwrap();
        rsfm::parser::parse_values(rsfm).unwrap()
    })
}

#[test]
fn correct_syntax() {
    let config = r#"
    var = 4

    rsfm.show_hidden = false
    rsfm.entry_format = {
        {
            type = "name",
            width = 5,
            is_fixed_width = false,
        },
        {
            type = "size",
            width = 5,
            is_fixed_width = true,
        }
    }
    "#;

    assert!(parse_syntax(&config).is_ok());
}

#[test]
fn no_rsfm() {
    let config = r#"
    var = 4
    "#;

    assert!(parse_syntax(&config).is_ok());
}

#[test]
fn unexpected_type() {
    let config = r#"
    rsfm.show_hidden = {
        x = 3
    }
    rsfm.entry_format = {
        {
            type = false
        }
    }
    "#;

    let result = parse_syntax(&config);
    assert!(result.is_err());

    let result = result.unwrap_err();
    assert_eq!(result.len(), 2);
    assert!(result
        .iter()
        .find(|&e| {
            e.eq("Unexpected type 'table' for variable 'rsfm.show_hidden', use 'boolean'")
        })
        .is_some(),);
    assert!(result
        .iter()
        .find(|&e| {
            e.eq("Unexpected type 'boolean' for variable 'rsfm.entry_format.1.type', use 'string'")
        })
        .is_some(),);
}

#[test]
fn unknown_variable() {
    let config = r#"
    rsfm.show_hidxxx = true
    rsfm.show_hixxxx = true
    rsfm.show_hidden = {
        false
    }
    rsfm.entry_format = {
        x = 3
    }
    rsfm.var = true
    "#;

    let result = parse_syntax(&config);
    assert!(result.is_err());

    let result = result.unwrap_err();
    println!("{:?}", result);
    assert_eq!(result.len(), 5);
    assert!(result
        .iter()
        .find(|&e| { e.eq("Unknown variable 'rsfm.show_hixxxx'") })
        .is_some(),);
    assert!(result
        .iter()
        .find(|&e| {
            e.eq("Unknown variable 'rsfm.show_hidxxx'. Did you mean 'rsfm.show_hidden'?")
        })
        .is_some(),);
    assert!(result
        .iter()
        .find(|&e| {
            e.eq("Unexpected type 'table' for variable 'rsfm.show_hidden', use 'boolean'")
        })
        .is_some(),);
    assert!(result
        .iter()
        .find(|&e| {
            e.eq("Unknown variable 'rsfm.entry_format.x'. Did you mean 'rsfm.entry_format.{}'?")
        })
        .is_some(),);
    assert!(result
        .iter()
        .find(|&e| { e.eq("Unknown variable 'rsfm.var'") })
        .is_some(),);
}

#[test]
fn unknown_variable_in_table() {
    let config = r#"
    rsfm.entry_format = {
        {
            x = 3
        }
    }
    "#;

    let result = parse_syntax(&config);
    assert!(result.is_err());

    let result = result.unwrap_err();
    assert_eq!(result.len(), 1);
    assert_eq!(
        result[0],
        "Unknown variable 'rsfm.entry_format.{}.x'. Did you mean 'rsfm.entry_format.{}'?"
    );

    let config = r#"
    rsfm.entry_format = {
        {
            hype = 3
        }
    }
    "#;

    let result = parse_syntax(&config);
    assert!(result.is_err());

    let result = result.unwrap_err();
    assert_eq!(result.len(), 1);
    assert_eq!(
        result[0],
        "Unknown variable 'rsfm.entry_format.{}.hype'. Did you mean 'rsfm.entry_format.{}.type'?"
    );
}

#[test]
fn correct_values() {
    let config = r#"
    rsfm.show_hidden = false
    rsfm.entry_format = {
        {
            type = "name",
            width = 5,
            is_fixed_width = false,
        },
        {
            type = "SiZe",
            width = 50,
            is_fixed_width = true,
        }
    }
    "#;

    let view_options = parse_values(&config);
    assert_eq!(view_options.show_hidden, false);
    assert_eq!(view_options.entry_format.len(), 2);

    let expected_0 = column::Column {
        column_type: column::ColumnType::from("name").unwrap(),
        width: 5,
        is_fixed_width: false,
    };
    let expected_1 = column::Column {
        column_type: column::ColumnType::from("size").unwrap(),
        width: 50,
        is_fixed_width: true,
    };
    assert_eq!(view_options.entry_format[0], expected_0);
    assert_eq!(view_options.entry_format[1], expected_1);
}

#[test]
fn entry_format_incomplete() {
    let config = r#"
    rsfm.show_hidden = false
    rsfm.entry_format = {
        {
            type = "name",
            width = 5,
        }
    }
    "#;

    let view_options = parse_values(&config);
    assert_eq!(view_options.show_hidden, false);
    assert_eq!(view_options.entry_format.len(), 0);
}

#[test]
fn entry_format_zero_width() {
    let config = r#"
    rsfm.show_hidden = false
    rsfm.entry_format = {
        {
            type = "name",
            width = 0,
            is_fixed_width = false,
        }
    }
    "#;

    let view_options = parse_values(&config);
    assert_eq!(view_options.show_hidden, false);
    assert_eq!(view_options.entry_format.len(), 0);
}

#[test]
fn entry_format_wrong_type() {
    let config = r#"
    rsfm.show_hidden = false
    rsfm.entry_format = {
        {
            type = "type",
            width = 10,
            is_fixed_width = false,
        }
    }
    "#;

    let view_options = parse_values(&config);
    assert_eq!(view_options.show_hidden, false);
    assert_eq!(view_options.entry_format.len(), 0);
}
