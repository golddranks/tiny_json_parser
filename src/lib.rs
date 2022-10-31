//#![no_std]

use core::{
    fmt::{self, Debug, Display},
    str::from_utf8,
};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Error;

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error")
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Nesting {
    depth: usize,
}

impl Debug for Nesting {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.depth)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Buffer<'buf> {
    offset: usize,
    buffer: &'buf [u8],
}

impl Debug for Buffer<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> fmt::Result {
        let before_utf8 = &self.buffer[self.offset..];
        let after_utf8 = &self.buffer[..self.offset];
        let before = from_utf8(before_utf8)
            .unwrap_or_else(|e| from_utf8(&before_utf8[..e.valid_up_to()]).unwrap());
        let after = from_utf8(after_utf8)
            .unwrap_or_else(|e| from_utf8(&after_utf8[..e.valid_up_to()]).unwrap());
        write!(f, "{}☞{}", before, after)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Parser<'buf> {
    nesting: Nesting,
    json: Buffer<'buf>,
}

impl Debug for Parser<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Parser")
            .field("nesting", &self.nesting)
            .field("json", &self.json)
            .finish()
    }
}

pub fn parse(json: &[u8]) -> Parser<'_> {
    Parser {
        nesting: Nesting { depth: 0 },
        json: Buffer {
            offset: 0,
            buffer: json,
        },
    }
}

impl<'buf> Parser<'buf> {
    pub fn descend(&mut self) {
        self.nesting.depth += 1;
    }

    pub fn ascend(&mut self) {
        self.nesting.depth -= 1;
    }

    pub fn offset(&self) -> usize {
        self.json.offset
    }

    // INVARIANT B: if peek succeeds, one call to step followed by that also succeeds
    pub fn peek(&self) -> Result<u8> {
        self.json.buffer.get(self.offset()).copied().ok_or(Error)
    }

    // INVARIANT B: if peek succeeds, one call to step followed by that also succeeds
    pub fn step(&mut self) -> Result<()> {
        if self.offset() == self.json.buffer.len() {
            return Err(Error);
        }
        self.json.offset += 1;
        Ok(())
    }

    pub fn ascend_to(&mut self, depth: usize) -> Result<()> {
        while depth < self.nesting.depth {
            match self.peek()? {
                b'}' => self.ascend(),
                b']' => self.ascend(),
                b'{' => self.descend(),
                b'[' => self.descend(),
                b'"' => {
                    self.parse_string()?;
                    continue;
                }
                _ => (),
            }
            self.step()?;
        }
        Ok(())
    }

    pub fn value<'a>(&'a mut self) -> Result<Val<'a, 'buf>> {
        self.skip_ws();
        Val::from(self)
    }

    pub fn finalize(&mut self) -> Result<()> {
        self.ascend_to(0)?;
        self.skip_ws();
        if self.offset() != self.json.buffer.len() {
            return Err(Error);
        }
        Ok(())
    }

    fn skip_ws(&mut self) {
        while let Ok(b' ' | b'\r' | b'\n' | b'\t') = self.peek() {
            let _ = self.step(); // Never happens because week peeked
        }
    }

    pub fn parse_number(&mut self) -> Result<Number<'buf>> {
        let start = self.offset();

        if let b'-' = self.peek()? {
            self.step()?;
        };

        match self.peek()? {
            b'0' => self.step()?,
            b'1'..=b'9' => {
                self.step()?;
                while let Ok(b'0'..=b'9') = self.peek() {
                    self.step()?;
                }
            }
            _ => return Err(Error),
        }

        if let Ok(b'.') = self.peek() {
            self.step()?;
            if let b'0'..=b'9' = self.peek()? {
                self.step()?;
            } else {
                return Err(Error);
            }
            while let Ok(b'0'..=b'9') = self.peek() {
                self.step()?;
            }
        }

        if let Ok(b'e' | b'E') = self.peek() {
            self.step()?;
            if let b'+' | b'-' = self.peek()? {
                self.step()?;
            }
            if let b'0'..=b'9' = self.peek()? {
                self.step()?;
            } else {
                return Err(Error);
            }
            while let Ok(b'0'..=b'9') = self.peek() {
                self.step()?;
            }
        }

        Ok(Number {
            number: &self.json.buffer[start..self.offset()],
        })
    }

    pub fn parse_string(&mut self) -> Result<String<'buf>> {
        if let b'"' = self.peek()? {
            self.step()?;
        } else {
            return Err(Error);
        }
        let start = self.offset();
        loop {
            match self.peek()? {
                b'"' => {
                    let contents = &self.json.buffer[start..self.offset()];
                    self.step()?;
                    let validated = from_utf8(contents).map_err(|_| Error)?;
                    return Ok(String { string: validated });
                }
                b'\\' => {
                    self.step()?;
                    match self.peek()? {
                        b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't' => {
                            self.step()?;
                        }
                        b'u' => {
                            self.step()?;
                            for _ in 0..4 {
                                match self.peek()? {
                                    b'0'..=b'9' | b'A'..=b'F' | b'a'..=b'f' => {
                                        self.step()?;
                                    }
                                    _ => return Err(Error),
                                }
                            }
                        }
                        _ => return Err(Error),
                    }
                }
                // Don't allow control characters (0..32).
                // UTF-8 continuation bytes are always of form 10xxxxxx (128..192),
                // so they are unaffected.
                c if c < 32 => return Err(Error),
                _ => self.step()?,
            }
        }
    }

    pub fn parse_array<'a>(&'a mut self) -> Result<Array<'a, 'buf>> {
        if let b'[' = self.peek()? {
            self.step()?;
            self.descend();
            Ok(Array {
                start: self.json.offset,
                depth: self.nesting.depth,
                parser: self,
            })
        } else {
            Err(Error)
        }
    }

    pub fn parse_object<'a>(&'a mut self) -> Result<Object<'a, 'buf>> {
        if let b'{' = self.peek()? {
            self.step()?;
            self.descend();
            Ok(Object {
                start: self.json.offset,
                depth: self.nesting.depth,
                parser: self,
            })
        } else {
            Err(Error)
        }
    }

    pub fn parse_word<'a>(&'a mut self, word: &[u8]) -> Result<()> {
        if let Some(json) = self.json.buffer.get(self.json.offset..) {
            if json.starts_with(word) {
                self.json.offset += word.len();
                return Ok(());
            }
        }
        Err(Error)
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
    start: usize,
    depth: usize,
    parser: &'a mut Parser<'buf>,
}

impl<'a, 'buf> Array<'a, 'buf> {
    pub fn next<'b>(&'b mut self) -> Result<Option<Val<'b, 'buf>>> {
        if self.parser.offset() == self.start {
            self.parser.skip_ws();
            if let b']' = self.parser.peek()? {
                self.parser.step()?;
                self.parser.ascend();
                return Ok(None);
            }
        } else {
            self.parser.ascend_to(self.depth)?;
            self.parser.skip_ws();
            match self.parser.peek()? {
                b',' => {
                    self.parser.step()?;
                    self.parser.skip_ws();
                }
                b']' => {
                    self.parser.step()?;
                    self.parser.ascend();
                    return Ok(None);
                }
                _ => return Err(Error),
            }
        }
        Ok(Some(Val::from(self.parser)?))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Object<'a, 'buf> {
    start: usize,
    depth: usize,
    parser: &'a mut Parser<'buf>,
}

impl<'a, 'buf> Object<'a, 'buf> {
    pub fn next<'b>(&'b mut self) -> Result<Option<(Key<'buf>, Val<'b, 'buf>)>> {
        if self.parser.offset() == self.start {
            self.parser.skip_ws();
            if let b'}' = self.parser.peek()? {
                self.parser.step()?;
                self.parser.ascend();
                return Ok(None);
            }
        } else {
            self.parser.ascend_to(self.depth)?;
            self.parser.skip_ws();
            match self.parser.peek()? {
                b',' => {
                    self.parser.step()?;
                    self.parser.skip_ws();
                }
                b'}' => {
                    self.parser.step()?;
                    self.parser.ascend();
                    return Ok(None);
                }
                _ => return Err(Error),
            }
        }
        let key = Key {
            key: self.parser.parse_string()?.as_str(),
        };
        self.parser.skip_ws();
        if let b':' = self.parser.peek()? {
            self.parser.step()?;
        } else {
            return Err(Error);
        }
        self.parser.skip_ws();
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

// TODO: tests only
pub fn string(str: &str) -> String<'_> {
    String { string: str }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Val<'pstate, 'buf> {
    Null,
    Boolean(bool),
    String(String<'buf>),
    Number(Number<'buf>),
    Array(Array<'pstate, 'buf>),
    Object(Object<'pstate, 'buf>),
}

impl<'a, 'buf> Val<'a, 'buf> {
    pub fn from(parser: &'a mut Parser<'buf>) -> Result<Val<'a, 'buf>> {
        Ok(match parser.peek()? {
            b'n' => {
                parser.parse_word(b"null")?;
                Val::Null
            }
            b'f' => {
                parser.parse_word(b"false")?;
                Val::Boolean(false)
            }
            b't' => {
                parser.parse_word(b"true")?;
                Val::Boolean(true)
            }
            b'-' | b'0'..=b'9' => Val::Number(parser.parse_number()?),
            b'"' => Val::String(parser.parse_string()?),
            b'{' => Val::Object(parser.parse_object()?),
            b'[' => Val::Array(parser.parse_array()?),
            _ => return Err(Error),
        })
    }
}

/*
pub enum Seg<'a> {
    Key(&'a [u8]),
    Idx(usize),
}

pub fn query<'a, 'b, const N: usize>(json: &'a impl AsRef<[u8]>, path: [Seg<'b>; N]) -> () {
    ()
}
*/
