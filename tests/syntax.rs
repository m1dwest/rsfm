use rsfm::parser::CheckResult;

fn check(source: &str) -> CheckResult {
    rlua::Lua::new().context(|ctx| -> CheckResult {
        let rsfm = ctx.create_table().unwrap();
        let globals = ctx.globals();

        globals.set("rsfm", rsfm).unwrap();

        ctx.load(source).exec().unwrap();

        let rsfm = globals.get::<_, rlua::Value>("rsfm").unwrap();
        rsfm::parser::parse_syntax("rsfm", rsfm)
    })
}

#[test]
fn correct() {
    let config = r#"
    var = 4

    rsfm.show_hidden = false
    rsfm.entry_format = {
        {
            type = "name",
            size = 5,
            is_fixed_size = false,
        },
        {
            type = "size",
            size = 5,
            is_fixed_size = true,
        }
    }
    "#;

    assert!(check(&config).is_ok());
}

#[test]
fn no_rsfm() {
    let config = r#"
    var = 4
    "#;

    assert!(check(&config).is_ok());
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

    let result = check(&config);
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

    let result = check(&config);
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
fn unknown_variable_in_table_1() {
    let config = r#"
    rsfm.entry_format = {
        {
            x = 3
        }
    }
    "#;

    let result = check(&config);
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

    let result = check(&config);
    assert!(result.is_err());

    let result = result.unwrap_err();
    assert_eq!(result.len(), 1);
    assert_eq!(
        result[0],
        "Unknown variable 'rsfm.entry_format.{}.hype'. Did you mean 'rsfm.entry_format.{}.type'?"
    );
}
