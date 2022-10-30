use tiny_json_parser::{parse, Error, Val, key};

fn main() {
    let json0 = br#""#;
    let json1 = br#"null"#;
    let json2 = br#"true"#;
    let json3 = br#"false"#;
    let json4 = br#"1.0"#;
    let json13 = br#""test""#;
    let json5 = br#"[]"#;
    let json6 = br#"[null]"#;
    let json7 = br#"[null, true]"#;
    let json8 = br#"[null, true, false]"#;
    let json9 = br#"[null, [], false]"#;
    let json10 = br#"[null, [true], false]"#;
    let json11 = br#"[null, [true, true], false]"#;
    let json12 = br#"{"a": null, "b": {"c": true, "d": null}, "e": false}"#;

    let mut p = parse(json0);
    assert_eq!(p.val(), Err(Error));

    let mut p = parse(json1);
    assert_eq!(p.val(), Ok(Val::Null));

    let mut p = parse(json2);
    assert_eq!(p.val().unwrap(), Val::Boolean(true));

    let mut p = parse(json3);
    assert_eq!(p.val().unwrap(), Val::Boolean(false));

    let mut p = parse(json4);
    if let Val::Number(num) = p.val().unwrap() {
        assert_eq!(num.as_bytes(), b"1.0");
        assert_eq!(num.as_str(), "1.0");
    } else {
        panic!();
    }

    let mut p = parse(json13);
    if let Val::String(str) = p.val().unwrap() {
        assert_eq!(str.as_str(), "test");
    } else {
        panic!();
    }

    match parse(json5).val().unwrap() {
        Val::Array(mut a) => {
            assert_eq!(a.next().unwrap(), None);
        }
        _ => panic!(),
    };

    match parse(json6).val().unwrap() {
        Val::Array(mut a) => {
            assert_eq!(a.next().unwrap(), Some(Val::Null));
            assert_eq!(a.next().unwrap(), None);
        }
        _ => panic!(),
    };

    match parse(json7).val().unwrap() {
        Val::Array(mut a) => {
            assert_eq!(a.next().unwrap(), Some(Val::Null));
            assert_eq!(a.next().unwrap(), Some(Val::Boolean(true)));
            assert_eq!(a.next().unwrap(), None);
        }
        _ => panic!(),
    };

    match parse(json8).val() {
        Ok(Val::Array(mut a)) => {
            assert_eq!(a.next().unwrap(), Some(Val::Null));
            assert_eq!(a.next().unwrap(), Some(Val::Boolean(true)));
            assert_eq!(a.next().unwrap(), Some(Val::Boolean(false)));
            assert_eq!(a.next().unwrap(), None);
        }
        _ => panic!(),
    };

    match parse(json9).val() {
        Ok(Val::Array(mut a)) => {
            assert_eq!(a.next().unwrap(), Some(Val::Null));
            if let Ok(Some(Val::Array(mut b))) = a.next() {
                assert_eq!(b.next().unwrap(), None);
            } else {
                panic!();
            };
            assert_eq!(a.next().unwrap(), Some(Val::Boolean(false)));
            assert_eq!(a.next().unwrap(), None);
        }
        _ => panic!(),
    };

    match parse(json9).val() {
        Ok(Val::Array(mut a)) => {
            assert_eq!(a.next().unwrap(), Some(Val::Null));
            let Ok(Some(Val::Array(_))) = a.next() else {
                panic!();
            };
            assert_eq!(a.next().unwrap(), Some(Val::Boolean(false)));
            assert_eq!(a.next().unwrap(), None);
        }
        _ => panic!(),
    };

    match parse(json10).val() {
        Ok(Val::Array(mut a)) => {
            assert_eq!(a.next().unwrap(), Some(Val::Null));
            if let Ok(Some(Val::Array(mut b))) = a.next() {
                assert_eq!(b.next().unwrap(), Some(Val::Boolean(true)));
                assert_eq!(b.next().unwrap(), None);
            } else {
                panic!();
            };
            assert_eq!(a.next().unwrap(), Some(Val::Boolean(false)));
            assert_eq!(a.next().unwrap(), None);
        }
        _ => panic!(),
    };

    match parse(json10).val() {
        Ok(Val::Array(mut a)) => {
            assert_eq!(a.next().unwrap(), Some(Val::Null));
            let Ok(Some(Val::Array(_))) = a.next() else {
                panic!();
            };
            assert_eq!(a.next().unwrap(), Some(Val::Boolean(false)));
            assert_eq!(a.next().unwrap(), None);
        }
        _ => panic!(),
    };

    match parse(json11).val() {
        Ok(Val::Array(mut a)) => {
            assert_eq!(a.next().unwrap(), Some(Val::Null));
            if let Ok(Some(Val::Array(mut b))) = a.next() {
                assert_eq!(b.next().unwrap(), Some(Val::Boolean(true)));
                assert_eq!(b.next().unwrap(), Some(Val::Boolean(true)));
                assert_eq!(b.next().unwrap(), None);
            } else {
                panic!();
            };
            assert_eq!(a.next().unwrap(), Some(Val::Boolean(false)));
        }
        _ => panic!(),
    };

    match parse(json11).val() {
        Ok(Val::Array(mut a)) => {
            assert_eq!(a.next(), Ok(Some(Val::Null)));
            let Ok(Some(Val::Array(_))) = a.next() else {
                panic!();
            };
            assert_eq!(a.next(), Ok(Some(Val::Boolean(false))));
        }
        _ => panic!(),
    };

    match parse(json12).val() {
        Ok(Val::Object(mut o)) => {
            assert_eq!(o.next(), Ok(Some((key("a"), Val::Null))));
            if let Ok(Some((k, Val::Object(mut p)))) = o.next() {
                assert_eq!(k, key("b"));
                assert_eq!(p.next().unwrap(), Some((key("c"), Val::Boolean(true))));
                assert_eq!(p.next().unwrap(), Some((key("d"), Val::Null)));
                assert_eq!(p.next().unwrap(), None);
            } else {
                panic!();
            };
            assert_eq!(o.next(), Ok(Some((key("e"), Val::Boolean(false)))));
        }
        _ => panic!(),
    };
}
