#[link(name = "xml", vers = "0.1", author = "Florob")];

#[ crate_type = "lib" ];
#[forbid(non_camel_case_types)];

use std::str;

#[deriving(Clone)]
pub enum XML {
    Element(~Element),
    CharacterNode(~str),
    CDATANode(~str),
    CommentNode(~str),
    PINode(~str)
}

#[deriving(Clone)]
pub struct Element {
    name: ~str,
    attributes: ~[Attribute],
    children: ~[XML]
}

#[deriving(Clone)]
pub struct Attribute {
    name: ~str,
    value: ~str
}

pub enum Event {
    PI(~str),
    StartTag { name: ~str, attributes: ~[Attribute] },
    EndTag { name: ~str },
    Characters(~str),
    CDATA(~str),
    Comment(~str),
    Null
}

#[deriving(Eq)]
/// If an error occurs while parsing some XML, this is the structure which is
/// returned
pub struct Error {
    /// The line number at which the error occurred
    line: uint,
    /// The column number at which the error occurred
    col: uint,
    /// A message describing the type of the error
    msg: @~str
}

// General functions
pub fn escape(input: &str) -> ~str {
    let tmp = str::replace(input, "&", "&amp;");
    let tmp = str::replace(tmp, "<", "&lt;");
    let tmp = str::replace(tmp, ">", "&gt;");
    let tmp = str::replace(tmp, "'", "&apos;");
    str::replace(tmp, "\"", "&quot;")
}

pub fn unescape(input: &str) -> ~str {
    let tmp = str::replace(input, "&quot;", "\"");
    let tmp = str::replace(tmp, "&apos;", "'");
    let tmp = str::replace(tmp, "&gt;", ">");
    let tmp = str::replace(tmp, "&lt;", "<");
    str::replace(tmp, "&amp;", "&")
}

// Event based parser
enum State {
    OutsideTag,
    TagOpened,
    InProcessingInstructions,
    InTagName,
    InCloseTagName,
    InTag,
    InAttrName,
    InAttrValue,
    ExpectDelimiter,
    ExpectClose,
    ExpectSpaceOrClose,
    InExclamationMark,
    InCDATAOpening,
    InCDATA,
    InCommentOpening,
    InComment1,
    InComment2,
    InDoctype
}

pub struct Parser {
    priv line: uint,
    priv col: uint,
    priv data: ~str,
    priv buf: ~str,
    priv name: ~str,
    priv attrName: ~str,
    priv attributes: ~[Attribute],
    priv delim: char,
    priv st: State,
    priv level: uint
}

pub fn Parser() -> Parser {
    let p = Parser {
        line: 1,
        col: 0,
        data: ~"",
        buf: ~"",
        name: ~"",
        attrName: ~"",
        attributes: ~[],
        delim: 0 as char,
        st: OutsideTag,
        level: 0
    };
    p
}

impl Parser {
    pub fn push_str(&mut self, buf: &str) {
        self.data.push_str(buf);
    }
    pub fn parse(&mut self) -> Result<Event, Error> {
        while self.data.len() > 0 {
            let c = self.data.shift_char();
            if c == '\n' {
                self.line += 1u;
                self.col = 0u;
            } else {
                self.col += 1u;
            }

            match self.parse_character(c) {
                Ok(Null) => loop,
                Err(e) => return Err(e),
                Ok(event) => return Ok(event)
            }
        }
        Ok(Null)
    }
}

impl Parser {
    fn error(&self, msg: ~str) -> Result<Event, Error> {
        Err(Error { line: self.line, col: self.col, msg: @msg })
    }

    fn parse_character(&mut self, c: char) -> Result<Event, Error> {
        // println(fmt!("Now in state: %?", self.st));
        match self.st {
            OutsideTag => self.outside_tag(c),
            TagOpened => self.tag_opened(c),
            InProcessingInstructions => self.in_processing_instructions(c),
            InTagName => self.in_tag_name(c),
            InCloseTagName => self.in_close_tag_name(c),
            InTag => self.in_tag(c),
            InAttrName => self.in_attr_name(c),
            InAttrValue => self.in_attr_value(c),
            ExpectDelimiter => self.expect_delimiter(c),
            ExpectClose => self.expect_close(c),
            ExpectSpaceOrClose => self.expect_space_or_close(c),
            InExclamationMark => self.in_exclamation_mark(c),
            InCDATAOpening => self.in_CDATA_opening(c),
            InCDATA => self.in_CDATA(c),
            InCommentOpening => self.in_comment_opening(c),
            InComment1 => self.in_comment1(c),
            InComment2 => self.in_comment2(c),
            InDoctype => self.in_doctype(c),
        }
    }

    fn outside_tag(&mut self, c: char) -> Result<Event, Error> {
        match c {
            '<' if self.buf.len() > 0 => {
                self.st = TagOpened;
                let buf = unescape(self.buf);
                self.buf = ~"";
                return Ok(Characters(buf));
            }
            '<' => self.st = TagOpened,
            _ => self.buf.push_char(c)
        }
        Ok(Null)
    }

    fn tag_opened(&mut self, c: char) -> Result<Event, Error> {
        self.st = match c {
            '?' => InProcessingInstructions,
            '!' => InExclamationMark,
            '/' => InCloseTagName,
            _ => {
                self.buf.push_char(c);
                InTagName
            }
        };
        Ok(Null)
    }

    fn in_processing_instructions(&mut self, c: char) -> Result<Event, Error> {
        match c {
            '?' => {
                self.level = 1;
                self.buf.push_char(c);
            }
            '>' if self.level == 1 => {
                self.level = 0;
                self.st = OutsideTag;
                let buf = self.buf.slice_chars(0, self.buf.char_len()-1).to_owned();
                self.buf = ~"";
                return Ok(PI(buf));
            }
            _ => self.buf.push_char(c)
        }
        Ok(Null)
    }

    fn in_tag_name(&mut self, c: char) -> Result<Event, Error> {
        match c {
            '/' | '>' => {
                self.st = if c == '/' {
                    ExpectClose
                } else {
                    OutsideTag
                };
                self.name = self.buf.clone();
                self.buf = ~"";
                let name = self.name.clone();
                return Ok(StartTag { name: name, attributes: ~[] });
            }
            ' ' | '\t' | '\r' | '\n' => {
                self.name = self.buf.clone();
                self.buf = ~"";
                self.st = InTag;
            }
            _ => self.buf.push_char(c)
        }
        Ok(Null)
    }

    fn in_close_tag_name(&mut self, c: char) -> Result<Event, Error> {
        match c {
            ' ' | '\t' | '\r' | '\n' | '>' => {
                let buf = self.buf.clone();
                self.buf = ~"";
                self.st = if c == '>' {
                    OutsideTag
                } else {
                    ExpectSpaceOrClose
                };
                Ok(EndTag { name: buf })
            }
            _ => {
                self.buf.push_char(c);
                Ok(Null)
            }
        }
    }

    fn in_tag(&mut self, c: char) -> Result<Event, Error> {
        match c {
            '/' | '>' => {
                let name = self.name.clone();
                self.st = if c == '/' {
                    ExpectClose
                } else {
                    self.name = ~"";
                    OutsideTag
                };
                let attr = self.attributes.clone();
                self.attributes = ~[];
                return Ok(StartTag { name: name, attributes: attr });
            }
            ' ' | '\t' | '\r' | '\n' => (),
            _ => {
                self.buf.push_char(c);
                self.st = InAttrName;
            }
        }
        Ok(Null)
    }

    fn in_attr_name(&mut self, c: char) -> Result<Event, Error> {
        match c {
            '=' => {
                self.level = 0;
                self.attrName = self.buf.clone();
                self.buf = ~"";
                self.st = ExpectDelimiter;
            }
            ' ' | '\t' | '\r' | '\n' => self.level = 1,
            _ if self.level == 0 => self.buf.push_char(c),
            _ => return self.error(~"Space occured in attribute name")
        }
        Ok(Null)
    }

    fn in_attr_value(&mut self, c: char) -> Result<Event, Error> {
        if c == self.delim {
            self.delim = 0 as char;
            self.st = InTag;
            let name = self.attrName.clone();
            self.attrName = ~"";
            let value = unescape(self.buf);
            self.buf = ~"";
            self.attributes.push(Attribute { name: name, value: value });
        } else {
            self.buf.push_char(c);
        }
        Ok(Null)
    }

    fn expect_delimiter(&mut self, c: char) -> Result<Event, Error> {
        match c {
            '"' | '\'' => {
                self.delim = c;
                self.st = InAttrValue;
            }
            ' ' | '\t' | '\r' | '\n' => (),
            _ => return self.error(~"Attribute value not enclosed in ' or \"")
        }
        Ok(Null)
    }

    fn expect_close(&mut self, c: char) -> Result<Event, Error> {
        match c {
            '>' => {
                self.st = OutsideTag;
                let name = self.name.clone();
                self.name = ~"";
                Ok(EndTag { name: name })
            }
            _ => self.error(~"Expected '>' to close tag")
       }
    }

    fn expect_space_or_close(&mut self, c: char) -> Result<Event, Error> {
        match c {
            ' ' | '\t' | '\r' | '\n' => Ok(Null),
            '>' => {
                self.st = OutsideTag;
                Ok(Null)
            }
            _ => self.error(~"Expected '>' to close tag, or LWS")
       }
    }

    fn in_exclamation_mark(&mut self, c: char) -> Result<Event, Error> {
        self.st = match c {
            '-' => InCommentOpening,
            '[' => InCDATAOpening,
            'D' => InDoctype,
            _ => return self.error(~"Malformed XML")
        };
        Ok(Null)
    }

    fn in_CDATA_opening(&mut self, c: char) -> Result<Event, Error> {
        static CDATAPattern: [char, ..6] = ['C', 'D', 'A', 'T', 'A', '['];
        if c == CDATAPattern[self.level] {
            self.level += 1;
        } else {
            return self.error(~"Invalid CDATA opening sequence")
        }

        if self.level == 6 {
            self.level = 0;
            self.st = InCDATA;
        }
        Ok(Null)
    }

    fn in_CDATA(&mut self, c: char) -> Result<Event, Error> {
        match c {
            ']' => {
                self.buf.push_char(c);
                self.level += 1;
            }
            '>' if self.level >= 2 => {
                self.st = OutsideTag;
                self.level = 0;
                let buf = self.buf.slice_chars(0, self.buf.char_len()-2).to_owned();
                self.buf = ~"";
                return Ok(CDATA(buf))
            }
            _ => {
                self.buf.push_char(c);
                self.level = 0;
            }
        }
        Ok(Null)
    }

    fn in_comment_opening(&mut self, c: char) -> Result<Event, Error> {
        if c == '-' {
            self.st = InComment1;
            self.level = 0;
            Ok(Null)
        } else {
            self.error(~"Expected 2nd '-' to start comment")
        }
    }

    fn in_comment1(&mut self, c: char) -> Result<Event, Error> {
        if c == '-' {
            self.level += 1;
        } else {
            self.level = 0;
        }

        if self.level == 2 {
            self.level = 0;
            self.st = InComment2;
        }

        self.buf.push_char(c);

        Ok(Null)
    }

    fn in_comment2(&mut self, c: char) -> Result<Event, Error> {
        if c != '>' {
            self.error(~"Not more than one adjacent '-' allowed in a comment")
        } else {
            self.st = OutsideTag;
            let buf = self.buf.slice_chars(0, self.buf.char_len()-2).to_owned();
            self.buf = ~"";
            Ok(Comment(buf))
        }
    }

    fn in_doctype(&mut self, c: char) -> Result<Event, Error> {
        static DOCTYPEPattern: [char, ..6] = ['O', 'C', 'T', 'Y', 'P', 'E'];
        match self.level {
            0..5 => if c == DOCTYPEPattern[self.level] {
                self.level += 1;
            } else {
                return self.error(~"Invalid DOCTYPE");
            },
            6 => {
                match c {
                    ' ' | '\t' | '\r' | '\n' => (),
                    _ => return self.error(~"Invalid DOCTYPE")
                }
                self.level += 1;
            }
            _ if c == '>' => {
                self.level = 0;
                self.st = OutsideTag;
            }
            _ => ()
        }
        Ok(Null)
    }
}

// DOM Builder
pub struct ElementBuilder {
    priv stack: ~[~Element]
}

pub fn ElementBuilder() -> ElementBuilder {
    let e = ElementBuilder {
        stack: ~[]
    };
    e
}

impl ElementBuilder {
    pub fn push_event(&mut self, e: Event) -> Result<Option<Element>, Error> {
        match e {
            PI(cont) => {
                let l = self.stack.len();
                if l > 0 {
                    (*self.stack[l-1]).children.push(PINode(cont));
                }
                Ok(None)
            }
            StartTag { name, attributes } => {
                self.stack.push(~Element {
                    name: name.clone(),
                    attributes: attributes.clone(),
                    children: ~[]
                });

                Ok(None)
            }
            EndTag { name } => {
                if self.stack.len() == 0 {
                    return Err(Error { line: 0, col: 0, msg: @~"Elements not properly nested" });
                }
                let elem = self.stack.pop();
                let l = self.stack.len();
                if elem.name != name {
                    Err(Error { line: 0, col: 0, msg: @~"Elements not properly nested" })
                } else if l == 0 {
                    Ok(Some(*elem))
                } else {
                    (*self.stack[l-1]).children.push(Element(elem));
                    Ok(None)
                }
            }
            Characters(chars) => {
                let l = self.stack.len();
                if l > 0 {
                    (*self.stack[l-1]).children.push(CharacterNode(chars));
                }
                Ok(None)
            }
            CDATA(chars) => {
                let l = self.stack.len();
                if l > 0 {
                    (*self.stack[l-1]).children.push(CDATANode(chars));
                }
                Ok(None)
            }
            Comment(cont) => {
                let l = self.stack.len();
                if l > 0 {
                    (*self.stack[l-1]).children.push(CommentNode(cont));
                }
                Ok(None)
            }
            Null => Ok(None)
        }
    }
}
