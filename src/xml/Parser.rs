// RustyXML
// Copyright (c) 2013 Florian Zeitz
//
// This project is MIT licensed.
// Please see the COPYING file for more information.
//
// The parser herein is derived from OFXMLParser as included with
// ObjFW, Copyright (c) 2008-2013 Jonathan Schleifer.
// Permission to license this derived work under MIT license has been granted by ObjFW's author.

use super::{unescape, Attribute, Event, PI, StartTag, EndTag, Characters, CDATA, Comment, Error};
use std::hashmap::HashMap;

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

/// A streaming XML parser
pub struct Parser {
    priv line: uint,
    priv col: uint,
    priv buf: ~str,
    priv name: ~str,
    priv prefix: Option<~str>,
    priv namespaces: ~[HashMap<~str, ~str>],
    priv attr_name: ~str,
    priv attr_prefix: Option<~str>,
    priv attributes: ~[Attribute],
    priv delim: Option<char>,
    priv st: State,
    priv level: uint
}

impl Parser {
    /// Returns a new `Parser`
    pub fn new() -> Parser {
        let mut p = Parser {
            line: 1,
            col: 0,
            buf: ~"",
            name: ~"",
            prefix: None,
            namespaces: ~[HashMap::with_capacity(2)],
            attr_name: ~"",
            attr_prefix: None,
            attributes: ~[],
            delim: None,
            st: OutsideTag,
            level: 0
        };
        p.namespaces[0].swap(~"xml", ~"http://www.w3.org/XML/1998/namespace");
        p.namespaces[0].swap(~"xmlns", ~"http://www.w3.org/2000/xmlns/");
        p
    }

    /**
     * Parses the string `data`.
     * The callback `cb` is called for each `Event`, or `Error` generated while parsing
     * the string.
     *
     * ~~~
     * let mut p = Parser::new();
     * do p.parse_str("<a href='http://rust-lang.org'>Rust</a>") |event| {
     *     match event {
     *        [...]
     *     }
     * }
     * ~~~
     */
    pub fn parse_str(&mut self, data: &str, cb: |Result<Event, Error>|) {
        for c in data.chars() {
            if c == '\n' {
                self.line += 1u;
                self.col = 0u;
            } else {
                self.col += 1u;
            }

            match self.parse_character(c) {
                Ok(None) => continue,
                Err(e) => {
                    cb(Err(e));
                    return;
                }
                Ok(Some(event)) => {
                    cb(Ok(event));
                }
            }
        }
    }
}

#[inline]
fn parse_qname(qname: &str) -> (Option<~str>, ~str) {
    match qname.find(':') {
        None => {
            (None, qname.to_owned())
        },
        Some(i) => {
            (Some(qname.slice_to(i).to_owned()), qname.slice_from(i+1).to_owned())
        }
    }
}

impl Parser {
    fn namespace_for_prefix(&self, prefix: &~str) -> Option<~str> {
        for ns in self.namespaces.rev_iter() {
            match ns.find(prefix) {
                None => continue,
                Some(namespace) => {
                    if namespace.len() == 0 {
                        return None;
                    } else {
                        return Some(namespace.to_owned());
                    }
                }
            }
        }
        None
    }

    fn error(&self, msg: ~str) -> Result<Option<Event>, Error> {
        Err(Error { line: self.line, col: self.col, msg: msg })
    }

    fn parse_character(&mut self, c: char) -> Result<Option<Event>, Error> {
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

    fn outside_tag(&mut self, c: char) -> Result<Option<Event>, Error> {
        match c {
            '<' if self.buf.len() > 0 => {
                self.st = TagOpened;
                let buf = unescape(self.buf);
                self.buf.clear();
                return Ok(Some(Characters(buf)));
            }
            '<' => self.st = TagOpened,
            _ => self.buf.push_char(c)
        }
        Ok(None)
    }

    fn tag_opened(&mut self, c: char) -> Result<Option<Event>, Error> {
        self.st = match c {
            '?' => InProcessingInstructions,
            '!' => InExclamationMark,
            '/' => InCloseTagName,
            _ => {
                self.buf.push_char(c);
                InTagName
            }
        };
        Ok(None)
    }

    fn in_processing_instructions(&mut self, c: char) -> Result<Option<Event>, Error> {
        match c {
            '?' => {
                self.level = 1;
                self.buf.push_char(c);
            }
            '>' if self.level == 1 => {
                self.level = 0;
                self.st = OutsideTag;
                let buf = self.buf.slice_chars(0, self.buf.char_len()-1).to_owned();
                self.buf.clear();
                return Ok(Some(PI(buf)));
            }
            _ => self.buf.push_char(c)
        }
        Ok(None)
    }

    fn in_tag_name(&mut self, c: char) -> Result<Option<Event>, Error> {
        fn set_name(p: &mut Parser) {
            let (prefix, name) = parse_qname(p.buf);
            p.prefix = prefix;
            p.name = name;
            p.buf.clear();
        };

        match c {
            '/'
            | '>' => {
                set_name(self);
                let prefix = self.prefix.take();
                let ns = match prefix {
                    None => self.namespace_for_prefix(&~""),
                    Some(ref pre) => {
                        self.namespace_for_prefix(pre).or_else(|| {
                            fail!("Unbound prefix: '{}'", *pre)
                        })
                    }
                };

                self.namespaces.push(HashMap::new());
                self.st = if c == '/' {
                    ExpectClose
                } else {
                    OutsideTag
                };

                return Ok(Some(StartTag(StartTag {
                    name: self.name.clone(),
                    ns: ns,
                    prefix: prefix,
                    attributes: ~[]
                })));
            }
            ' '
            | '\t'
            | '\r'
            | '\n' => {
                self.namespaces.push(HashMap::new());
                set_name(self);
                self.st = InTag;
            }
            _ => self.buf.push_char(c)
        }
        Ok(None)
    }

    fn in_close_tag_name(&mut self, c: char) -> Result<Option<Event>, Error> {
        match c {
            ' '
            | '\t'
            | '\r'
            | '\n'
            | '>' => {
                let (prefix, name) = parse_qname(self.buf);
                self.buf.clear();

                let ns = match prefix {
                    None => self.namespace_for_prefix(&~""),
                    Some(ref pre) => {
                        self.namespace_for_prefix(pre).or_else(|| {
                            fail!("Unbound prefix: '{}'", *pre)
                        })
                    }
                };

                self.namespaces.pop();
                self.st = if c == '>' {
                    OutsideTag
                } else {
                    ExpectSpaceOrClose
                };

                Ok(Some(EndTag(EndTag { name: name, ns: ns, prefix: prefix })))
            }
            _ => {
                self.buf.push_char(c);
                Ok(None)
            }
        }
    }

    fn in_tag(&mut self, c: char) -> Result<Option<Event>, Error> {
        match c {
            '/'
            | '>' => {
                let name = self.name.clone();
                let mut attributes = self.attributes.clone();
                self.attributes = ~[];
                let prefix = self.prefix.clone();
                let ns = match prefix {
                    None => self.namespace_for_prefix(&~""),
                    Some(ref pre) => {
                        self.namespace_for_prefix(pre).or_else(|| {
                            fail!("Unbound prefix: '{}'", *pre)
                        })
                    }
                };

                for attr in attributes.mut_iter() {
                    attr.ns.mutate( |ref pre| {
                        self.namespace_for_prefix(pre).unwrap_or_else( || {
                            fail!("Unbound prefix: '{}'", *pre)
                        })
                    });
                }

                self.st = if c == '/' {
                    ExpectClose
                } else {
                    self.name.clear();
                    self.prefix = None;
                    OutsideTag
                };

                return Ok(Some(StartTag(StartTag {
                    name: name,
                    ns: ns,
                    prefix: prefix,
                    attributes: attributes
                })));
            }
            ' '
            | '\t'
            | '\r'
            | '\n' => (),
            _ => {
                self.buf.push_char(c);
                self.st = InAttrName;
            }
        }
        Ok(None)
    }

    fn in_attr_name(&mut self, c: char) -> Result<Option<Event>, Error> {
        match c {
            '=' => {
                self.level = 0;

                let (prefix, name) = parse_qname(self.buf);
                self.attr_prefix = prefix;
                self.attr_name = name;

                self.buf.clear();
                self.st = ExpectDelimiter;
            }
            ' '
            | '\t'
            | '\r'
            | '\n' => self.level = 1,
            _ if self.level == 0 => self.buf.push_char(c),
            _ => return self.error(~"Space occured in attribute name")
        }
        Ok(None)
    }

    fn in_attr_value(&mut self, c: char) -> Result<Option<Event>, Error> {
        if c == self.delim.expect("In attribute value, but no delimiter set") {
            self.delim = None;
            self.st = InTag;
            let name = self.attr_name.clone();
            self.attr_name.clear();
            let value = unescape(self.buf);
            self.buf.clear();
            let prefix = self.attr_prefix.clone();
            self.attr_prefix = None;

            match prefix {
                None if name.as_slice() == "xmlns" => {
                    let last = &mut self.namespaces[self.namespaces.len()-1];
                    last.swap(~"", value.clone());
                }
                Some(ref prefix) if prefix.as_slice() == "xmlns" => {
                    let last = &mut self.namespaces[self.namespaces.len()-1];
                    last.swap(name.clone(), value.clone());
                }
                _ => ()
            }

            self.attributes.push(Attribute { name: name, ns: prefix, value: value });
        } else {
            self.buf.push_char(c);
        }
        Ok(None)
    }

    fn expect_delimiter(&mut self, c: char) -> Result<Option<Event>, Error> {
        match c {
            '"'
            | '\'' => {
                self.delim = Some(c);
                self.st = InAttrValue;
            }
            ' '
            | '\t'
            | '\r'
            | '\n' => (),
            _ => return self.error(~"Attribute value not enclosed in ' or \"")
        }
        Ok(None)
    }

    fn expect_close(&mut self, c: char) -> Result<Option<Event>, Error> {
        match c {
            '>' => {
                self.st = OutsideTag;
                let name = self.name.clone();
                self.name.clear();
                let prefix = self.prefix.take();
                let ns = match prefix {
                    None => self.namespace_for_prefix(&~""),
                    Some(ref pre) => {
                        self.namespace_for_prefix(pre).or_else(|| {
                            fail!("Unbound prefix: '{}'", *pre)
                        })
                    }
                };
                self.namespaces.pop();
                Ok(Some(EndTag(EndTag { name: name, ns: ns, prefix: prefix })))
            }
            _ => self.error(~"Expected '>' to close tag")
       }
    }

    fn expect_space_or_close(&mut self, c: char) -> Result<Option<Event>, Error> {
        match c {
            ' '
            | '\t'
            | '\r'
            | '\n' => Ok(None),
            '>' => {
                self.st = OutsideTag;
                Ok(None)
            }
            _ => self.error(~"Expected '>' to close tag, or LWS")
       }
    }

    fn in_exclamation_mark(&mut self, c: char) -> Result<Option<Event>, Error> {
        self.st = match c {
            '-' => InCommentOpening,
            '[' => InCDATAOpening,
            'D' => InDoctype,
            _ => return self.error(~"Malformed XML")
        };
        Ok(None)
    }

    fn in_CDATA_opening(&mut self, c: char) -> Result<Option<Event>, Error> {
        static CDATA_PATTERN: [char, ..6] = ['C', 'D', 'A', 'T', 'A', '['];
        if c == CDATA_PATTERN[self.level] {
            self.level += 1;
        } else {
            return self.error(~"Invalid CDATA opening sequence")
        }

        if self.level == 6 {
            self.level = 0;
            self.st = InCDATA;
        }
        Ok(None)
    }

    fn in_CDATA(&mut self, c: char) -> Result<Option<Event>, Error> {
        match c {
            ']' => {
                self.buf.push_char(c);
                self.level += 1;
            }
            '>' if self.level >= 2 => {
                self.st = OutsideTag;
                self.level = 0;
                let buf = self.buf.slice_chars(0, self.buf.char_len()-2).to_owned();
                self.buf.clear();
                return Ok(Some(CDATA(buf)))
            }
            _ => {
                self.buf.push_char(c);
                self.level = 0;
            }
        }
        Ok(None)
    }

    fn in_comment_opening(&mut self, c: char) -> Result<Option<Event>, Error> {
        if c == '-' {
            self.st = InComment1;
            self.level = 0;
            Ok(None)
        } else {
            self.error(~"Expected 2nd '-' to start comment")
        }
    }

    fn in_comment1(&mut self, c: char) -> Result<Option<Event>, Error> {
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

        Ok(None)
    }

    fn in_comment2(&mut self, c: char) -> Result<Option<Event>, Error> {
        if c != '>' {
            self.error(~"Not more than one adjacent '-' allowed in a comment")
        } else {
            self.st = OutsideTag;
            let buf = self.buf.slice_chars(0, self.buf.char_len()-2).to_owned();
            self.buf.clear();
            Ok(Some(Comment(buf)))
        }
    }

    fn in_doctype(&mut self, c: char) -> Result<Option<Event>, Error> {
        static DOCTYPE_PATTERN: [char, ..6] = ['O', 'C', 'T', 'Y', 'P', 'E'];
        match self.level {
            0..5 => if c == DOCTYPE_PATTERN[self.level] {
                self.level += 1;
            } else {
                return self.error(~"Invalid DOCTYPE");
            },
            6 => {
                match c {
                    ' '
                    | '\t'
                    | '\r'
                    | '\n' => (),
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
        Ok(None)
    }
}
