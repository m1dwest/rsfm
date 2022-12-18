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
        "first:1", "second:2", "third:3"
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
    rsfm.entry_format = 3
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
            e.eq("Unexpected type 'integer' for variable 'rsfm.entry_format', use 'table'")
        })
        .is_some(),);
}

#[test]
fn unknown_variable() {
    let config = r#"
    rsfm.show_hidxxx = true
    rsfm.show_hixxxx = true
    rsfm.entry_format = {
        x = 3
    }
    rsfm.var = true
    "#;

    let result = check(&config);
    assert!(result.is_err());

    let result = result.unwrap_err();
    assert_eq!(result.len(), 4);
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
            e.eq("Unknown variable 'rsfm.entry_format.x'. Did you mean 'rsfm.entry_format{}'?")
        })
        .is_some(),);
    assert!(result
        .iter()
        .find(|&e| { e.eq("Unknown variable 'rsfm.var'") })
        .is_some(),);
}
