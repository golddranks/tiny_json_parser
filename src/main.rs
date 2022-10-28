use tiny_json_parser::{parse, Val, Error, Number, Array};

fn main() {
    let json0 = br#""#;
    let json1 = br#"null"#;
    let json2 = br#"true"#;
    let json3 = br#"false"#;
    let json4 = br#"1.0"#;
    let json5 = br#"[]"#;
    let json6 = br#"[null]"#;
    let json7 = br#"[null, true]"#;
    let json8 = br#"[null, true, false]"#;
    let json9 = br#"[null, [], false]"#;
    let json10 = br#"[null, [true], false]"#;
    let json11 = br#"[null, [true, true], false]"#;

    let mut p = parse(json0).unwrap();
    assert_eq!(p.val(), Err(Error));

    let mut p = parse(json1).unwrap();
    assert_eq!(p.val().unwrap(), Val::Null);

    let mut p = parse(json2).unwrap();
    assert_eq!(p.val().unwrap(), Val::Boolean(true));

    let mut p = parse(json3).unwrap();
    assert_eq!(p.val().unwrap(), Val::Boolean(false));

    let mut p = parse(json4).unwrap();
    if let Val::Number(num) = p.val().unwrap() {
        assert_eq!(num.as_bytes(), b"1.0");
    } else {
        panic!();
    }

    match parse(json5).unwrap().val().unwrap() {
        Val::Array(mut a) => {
            assert_eq!(a.next().unwrap(), None);
        },
        _ => panic!(),
    };

    match parse(json6).unwrap().val().unwrap() {
        Val::Array(mut a) => {
            assert_eq!(a.next().unwrap().unwrap(), Val::Null);
            assert_eq!(a.next().unwrap(), None);
        },
        _ => panic!(),
    };


    match parse(json7).unwrap().val().unwrap() {
        Val::Array(mut a) => {
            assert_eq!(a.next().unwrap().unwrap(), Val::Null);
            assert_eq!(a.next().unwrap().unwrap(), Val::Boolean(true));
            assert_eq!(a.next().unwrap(), None);
        },
        _ => panic!(),
    };

    match parse(json8).unwrap().val().unwrap() {
        Val::Array(mut a) => {
            assert_eq!(a.next().unwrap().unwrap(), Val::Null);
            assert_eq!(a.next().unwrap().unwrap(), Val::Boolean(true));
            assert_eq!(a.next().unwrap().unwrap(), Val::Boolean(false));
            assert_eq!(a.next().unwrap(), None);
        },
        _ => panic!(),
    };

    match parse(json9).unwrap().val().unwrap() {
        Val::Array(mut a) => {
            assert_eq!(a.next().unwrap().unwrap(), Val::Null);
            if let Val::Array(mut a) = a.next().unwrap().unwrap() {
                assert_eq!(a.next().unwrap(), None);
            } else {
                panic!();
            };
            assert_eq!(a.next().unwrap().unwrap(), Val::Boolean(false));
            assert_eq!(a.next().unwrap(), None);
        },
        _ => panic!(),
    };

    match parse(json9).unwrap().val().unwrap() {
        Val::Array(mut a) => {
            assert_eq!(a.next().unwrap().unwrap(), Val::Null);
            let Val::Array(_) = a.next().unwrap().unwrap() else {
                panic!();
            };
            assert_eq!(a.next().unwrap().unwrap(), Val::Boolean(false));
            assert_eq!(a.next().unwrap(), None);
        },
        _ => panic!(),
    };

    match parse(json10).unwrap().val().unwrap() {
        Val::Array(mut a) => {
            assert_eq!(a.next().unwrap().unwrap(), Val::Null);
            if let Val::Array(mut a) = a.next().unwrap().unwrap() {
                assert_eq!(a.next().unwrap().unwrap(), Val::Boolean(true));
                assert_eq!(a.next().unwrap(), None);
            } else {
                panic!();
            };
            assert_eq!(a.next().unwrap().unwrap(), Val::Boolean(false));
            assert_eq!(a.next().unwrap(), None);
        },
        _ => panic!(),
    };

    match parse(json10).unwrap().val().unwrap() {
        Val::Array(mut a) => {
            assert_eq!(a.next().unwrap().unwrap(), Val::Null);
            let Val::Array(_) = a.next().unwrap().unwrap() else {
                panic!();
            };
            assert_eq!(a.next().unwrap().unwrap(), Val::Boolean(false));
            assert_eq!(a.next().unwrap(), None);
        },
        _ => panic!(),
    };
}
