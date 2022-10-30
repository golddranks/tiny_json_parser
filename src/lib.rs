//#![no_std]

use core::{fmt::Display, str::from_utf8};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Error;

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parser<'buf> {
    offset: usize,
    depth: usize,
    json: &'buf [u8],
}

pub fn parse(json: &impl AsRef<[u8]>) -> Parser<'_> {
    Parser {
        offset: 0,
        depth: 0,
        json: json.as_ref(),
    }
}

impl<'buf> Parser<'buf> {
    pub fn descend(&mut self) {
        self.depth += 1;
    }

    pub fn ascend(&mut self) {
        self.depth -= 1;
    }

    pub fn parse_string(&mut self) -> Result<String<'buf>> {
        if let Some(b'"') = self.json.get(self.offset) {
            self.offset += 1;
        } else {
            return Err(Error);
        }
        let start = self.offset;
        loop {
            match self.json.get(self.offset) {
                Some(b'"') => {
                    let contents = &self.json[start..self.offset];
                    self.offset += 1;
                    let validated = from_utf8(contents).map_err(|_| Error)?;
                    return Ok(String { string: validated });
                },
                Some(b'\\') => {
                    self.offset += 1;
                    match self.json.get(self.offset) {
                        Some(b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't') => {
                            self.offset += 1;
                        }
                        Some(b'u') => {
                            self.offset += 1;
                            for _ in 0..4 {
                                match self.json.get(self.offset) {
                                    Some(b'0'..=b'9' | b'A'..=b'F' | b'a'..=b'f') => {
                                        self.offset += 1
                                    }
                                    Some(_) => return Err(Error),
                                    None => return Err(Error),
                                }
                            }
                        }
                        Some(_) => return Err(Error),
                        None => (),
                    }
                },
                // Don't allow control characters (0..32).
                // UTF-8 continuation bytes are always of form 10xxxxxx (128..192),
                // so they are unaffected.
                Some(&c) if c < 32 => return Err(Error),
                Some(_) => self.offset += 1,
                None => return Err(Error),
            }
        }
    }

    pub fn ascend_to(&mut self, depth: usize) -> Result<()> {
        while depth < self.depth {
            match self.json.get(self.offset) {
                Some(b'}') => self.depth -= 1,
                Some(b']') => self.depth -= 1,
                Some(b'{') => self.depth += 1,
                Some(b'[') => self.depth += 1,
                Some(b'"') => {
                    self.parse_string()?;
                }
                Some(_) => (),
                None => return Err(Error),
            }
            self.offset += 1;
        }
        Ok(())
    }

    pub fn val<'a>(&'a mut self) -> Result<Val<'a, 'buf>> {
        Val::from(self)
    }

    fn skip_ws(&mut self) {
        while let Some(b' ' | b'\r' | b'\n' | b'\t') = self.json.get(self.offset) {
            self.offset += 1;
        }
    }

    pub fn parse_number(&mut self) -> Result<Number<'buf>> {
        let start = self.offset;

        if let Some(b'-') = self.json.get(self.offset) {
            self.offset += 1;
            true
        } else {
            false
        };

        match self.json.get(self.offset) {
            Some(b'0') => self.offset += 1,
            Some(b'1'..=b'9') => {
                self.offset += 1;
                while let Some(b'0'..=b'9') = self.json.get(self.offset) {
                    self.offset += 1;
                }
            }
            _ => return Err(Error),
        }

        if let Some(b'.') = self.json.get(self.offset) {
            self.offset += 1;
            if let Some(b'0'..=b'9') = self.json.get(self.offset) {
                self.offset += 1;
            } else {
                return Err(Error);
            }
            while let Some(b'0'..=b'9') = self.json.get(self.offset) {
                self.offset += 1;
            }
        }

        if let Some(b'e' | b'E') = self.json.get(self.offset) {
            self.offset += 1;
            if let Some(b'+' | b'-') = self.json.get(self.offset) {
                self.offset += 1;
            }
            if let Some(b'0'..=b'9') = self.json.get(self.offset) {
                self.offset += 1;
            } else {
                return Err(Error);
            }
            while let Some(b'0'..=b'9') = self.json.get(self.offset) {
                self.offset += 1;
            }
        }

        Ok(Number {
            number: &self.json[start..self.offset],
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Number<'buf> {
    // INVARIANT A:
    // `Number.number` can only ever contain bytes b'-', b'+', b'0'..b'9', b'.', b'e', b'E'.
    number: &'buf [u8],
}

impl<'buf> Number<'buf> {
    pub fn as_bytes(&self) -> &'buf [u8] {
        self.number
    }

    pub fn as_str(&self) -> &'buf str {
        // Never panics because of INVARIANT A
        from_utf8(self.number).unwrap()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct String<'buf> {
    string: &'buf str,
}

impl<'buf> String<'buf> {
    pub fn as_str(&self) -> &'buf str {
        self.string
    }

    pub fn as_bytes(&self) -> &'buf [u8] {
        self.string.as_bytes()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Array<'a, 'buf> {
    depth: usize,
    parser: &'a mut Parser<'buf>,
}

impl<'a, 'buf> Array<'a, 'buf> {
    pub fn next<'b>(&'b mut self) -> Result<Option<Val<'b, 'buf>>> {
        self.parser.ascend_to(self.depth)?;
        self.parser.skip_ws();
        match self.parser.json.get(self.parser.offset) {
            Some(b',') => {
                self.parser.offset += 1;
            }
            Some(b']') => {
                self.parser.offset += 1;
                self.parser.ascend();
                return Ok(None);
            }
            _ => (),
        }
        Ok(Some(Val::from(self.parser)?))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Object<'a, 'buf> {
    depth: usize,
    parser: &'a mut Parser<'buf>,
}

impl<'a, 'buf> Object<'a, 'buf> {
    pub fn next<'b>(&'b mut self) -> Result<Option<(Key<'buf>, Val<'b, 'buf>)>> {
        self.parser.ascend_to(self.depth)?;
        self.parser.skip_ws();
        match self.parser.json.get(self.parser.offset) {
            Some(b',') => {
                self.parser.offset += 1;
            }
            Some(b'}') => {
                self.parser.offset += 1;
                self.parser.ascend();
                return Ok(None);
            }
            _ => (),
        }
        self.parser.skip_ws();
        let key = Key { key: self.parser.parse_string()?.as_str() };
        self.parser.skip_ws();
        if let Some(b':') = self.parser.json.get(self.parser.offset) {
            self.parser.offset += 1;
        }
        let val = Val::from(self.parser)?;
        Ok(Some((key, val)))
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Key<'a> {
    key: &'a str,
}

// TODO: tests only
pub fn key(key: &str) -> Key {
    Key { key }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Val<'a, 'buf> {
    Null,
    Boolean(bool),
    String(String<'buf>),
    Number(Number<'buf>),
    Array(Array<'a, 'buf>),
    Object(Object<'a, 'buf>),
}

impl<'a, 'buf> Val<'a, 'buf> {
    pub fn from(parser: &'a mut Parser<'buf>) -> Result<Val<'a, 'buf>> {
        parser.skip_ws();
        let json = &parser.json[parser.offset..];
        Ok(match json {
            _ if json.starts_with(b"null") => {
                parser.offset += 4;
                Val::Null
            }
            _ if json.starts_with(b"false") => {
                parser.offset += 5;
                Val::Boolean(false)
            }
            _ if json.starts_with(b"true") => {
                parser.offset += 4;
                Val::Boolean(true)
            }
            [b'-' | b'0'..=b'9', ..] => Val::Number(parser.parse_number()?),
            [b'"', ..] => Val::String(parser.parse_string()?),
            [b'{', ..] => {
                parser.offset += 1;
                parser.descend();
                Val::Object(Object {
                    depth: parser.depth,
                    parser,
                })
            }
            [b'[', ..] => {
                parser.offset += 1;
                parser.descend();
                Val::Array(Array {
                    depth: parser.depth,
                    parser,
                })
            }
            _ => return Err(Error), // TODO: proper error handling
        })
    }
}

pub enum Seg<'a> {
    Key(&'a [u8]),
    Idx(usize),
}

pub fn query<'a, 'b, const N: usize>(json: &'a impl AsRef<[u8]>, path: [Seg<'b>; N]) -> () {
    ()
}
