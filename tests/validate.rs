use std::{
    error::Error,
    fs::read_dir,
    path::{Path, PathBuf},
};
use tiny_json_parser::{parse, Error as JsonError, Val};

fn validate(json: &[u8]) -> Result<(), JsonError> {
    let mut p = parse(json);
    match p.value()? {
        Val::Null => (),
        Val::Boolean(_) => (),
        Val::String(_) => (),
        Val::Number(_) => (),
        Val::Array(mut a) => while let Some(_) = a.next()? {},
        Val::Object(mut o) => while let Some(_) = o.next()? {},
    }
    p.finalize()?;

    let mut p = parse(json);
    p.value()?;
    p.finalize()?;

    Ok(())
}

fn validate_dir(path: impl AsRef<Path>) -> Result<(), Box<dyn Error + 'static>> {
    let mut dir = Vec::new();
    for entry in read_dir(path.as_ref())? {
        dir.push(entry?.file_name());
    }
    dir.sort();
    let mut pathbuf = PathBuf::from(path.as_ref());
    let mut errors = 0;
    for fname in dir {
        pathbuf.push(&fname);
        let fname = fname.to_string_lossy();
        if fname.ends_with(".json") {
            let expect = &fname[0..1];
            let json = std::fs::read(&pathbuf)?;
            match (expect, validate(&json)) {
                ("y", Ok(())) => println!("OK\t{}", fname),
                ("y", Err(_)) => {
                    errors += 1;
                    println!("ERR\t{}", fname)
                }
                ("n", Ok(())) => {
                    errors += 1;
                    println!("ERR\t{}", fname)
                }
                ("n", Err(_)) => println!("OK\t{}", fname),
                ("i", Ok(())) => println!("?\t{}", fname),
                ("i", Err(_)) => println!("?\t{}", fname),
                (_, _) => println!("Unexpected file: {}", fname),
            }
        }
        pathbuf.pop();
    }

    if errors == 0 {
        Ok(())
    } else {
        Err(format!(" {} errors", errors))?
    }
}

#[test]
fn minefield() -> Result<(), Box<dyn Error + 'static>> {
    validate_dir("tests/minefield")?;
    Ok(())
}

#[test]
fn kontio() -> Result<(), Box<dyn Error + 'static>> {
    validate_dir("tests/kontio")?;
    Ok(())
}
