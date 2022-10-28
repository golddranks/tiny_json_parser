//#![no_std]

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parser<'buf> {
    offset: usize,
    json: &'buf [u8],
}

pub fn parse(json: &impl AsRef<[u8]>) -> Result<Parser<'_>> {
    Ok(Parser {
        offset: 0,
        json: json.as_ref()
    })
}

impl<'buf> Parser<'buf> {
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

        let negative = if let Some(b'-') = self.json.get(self.offset) {
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
            },
            _ => return Err(Error),
        }

        if let Some(b'.') = self.json.get(self.offset) {
            self.offset += 1;
            if let Some(b'0'..=b'9') = self.json.get(self.offset) {
                self.offset += 1;
            } else {
                return Err(Error)
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
                return Err(Error)
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
    number: &'buf [u8],
}

impl Number<'_> {
    pub fn as_bytes(&self) -> &[u8] {
        self.number
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Array<'a, 'buf> {
    parser: &'a mut Parser<'buf>,
}

impl<'a, 'buf> Array<'a, 'buf> {
    pub fn next<'b>(&'b mut self) -> Result<Option<Val<'b, 'buf>>> {
        self.parser.skip_ws();
        match self.parser.json.get(self.parser.offset) {
            Some(b',') => {
                self.parser.offset += 1;
            },
            Some(b']') => {
                self.parser.offset += 1;
                return Ok(None)
            },
            _ => (),
        }
        Ok(Some(Val::from(self.parser)?))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Object<'a, 'buf> {
    parser: &'a mut Parser<'buf>,
}
/*
impl<'buf> Object<'buf> {
    fn next<'a>(&'a mut self) -> Result<(Key<'_>, Val<'a, 'buf>), Error> {
        let json = self.parser.json;
        let (key, tail) = Key::from(self.json)?;
        let val = Val::from(tail)?;
        self.json = tail;
        Ok((key, val))
    }
}
 */

pub struct Key<'a> {
    key: &'a [u8],
}

impl Key<'_> {
    fn from(input: &[u8]) -> Result<(Key<'_>, &[u8])> {

        Ok((Key{ key: &[] }, &[]))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Val<'a, 'buf> {
    Null,
    Boolean(bool),
    String,
    Number(Number<'buf>),
    Array(Array<'a, 'buf>),
    Object(Object<'a, 'buf>),
}

impl<'a, 'buf> Val<'a, 'buf> {
    pub fn from(parser: &'a mut Parser<'buf>) -> Result<Val<'a, 'buf>> {
        // TODO: consume whitespace
        parser.skip_ws();
        let json= &parser.json[parser.offset..];
        dbg!(std::str::from_utf8(json));
        Ok(match json {
            [b'{', ..] => {
                parser.offset += 1;
                Val::Object(Object { parser })
            },
            [b'[', ..] => {
                parser.offset += 1;
                Val::Array(Array { parser })
            },
            _ if json.starts_with(b"null") => {
                parser.offset += 4;
                Val::Null
            },
            _ if json.starts_with(b"false") => {
                parser.offset += 5;
                Val::Boolean(false)
            },
            _ if json.starts_with(b"true") => {
                parser.offset += 4;
                Val::Boolean(true)
            },
            [b'-' | b'0'..=b'9', ..] => {
                Val::Number(parser.parse_number()?)
            },
            // TODO: strings
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
