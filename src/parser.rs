// RustyXML
// Copyright 2013-2016 RustyXML developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//
// The parser herein is derived from OFXMLParser as included with
// ObjFW, Copyright (c) 2008-2013 Jonathan Schleifer.
// Permission to license this derived work under MIT license has been granted by ObjFW's author.

use super::{unescape, EndTag, StartTag};
use std::collections::{HashMap, VecDeque};
use std::error::Error;
use std::fmt;
use std::iter::Iterator;
use std::mem;

#[derive(PartialEq, Eq, Debug)]
/// Events returned by the `Parser`
pub enum Event {
    /// Event indicating processing information was found
    PI(String),
    /// Event indicating a start tag was found
    ElementStart(StartTag),
    /// Event indicating a end tag was found
    ElementEnd(EndTag),
    /// Event indicating character data was found
    Characters(String),
    /// Event indicating CDATA was found
    CDATA(String),
    /// Event indicating a comment was found
    Comment(String),
}

#[derive(PartialEq, Debug, Clone)]
#[allow(missing_copy_implementations)]
/// The structure returned, when erroneous XML is read
pub struct ParserError {
    /// The line number at which the error occurred
    pub line: u32,
    /// The column number at which the error occurred
    pub col: u32,
    /// A message describing the type of the error
    pub msg: &'static str,
}

impl Error for ParserError {
    fn description(&self) -> &str {
        self.msg
    }
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Parse error; Line: {}, Column: {}, Reason: {}",
            self.line, self.col, self.msg,
        )
    }
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
    InDoctype,
}

/// A streaming XML parser
///
/// Data is fed to the parser using the `feed_str()` method.
/// The `Event`s, and `ParserError`s generated while parsing the string
/// can be requested by iterating over the parser
///
/// ~~~
/// use xml::Parser;
///
/// let mut p = Parser::new();
/// p.feed_str("<a href='http://rust-lang.org'>Rust</a>");
/// for event in p {
///     match event {
///        // [...]
///        _ => ()
///     }
/// }
/// ~~~
pub struct Parser {
    line: u32,
    col: u32,
    has_error: bool,
    data: VecDeque<char>,
    buf: String,
    namespaces: Vec<HashMap<String, String>>,
    attributes: Vec<(String, Option<String>, String)>,
    st: State,
    name: Option<(Option<String>, String)>,
    attr: Option<(Option<String>, String)>,
    delim: Option<char>,
    level: u8,
}

impl Parser {
    /// Returns a new `Parser`
    pub fn new() -> Parser {
        let mut ns = HashMap::with_capacity(2);
        // Add standard namespaces
        ns.insert(
            "xml".to_owned(),
            "http://www.w3.org/XML/1998/namespace".to_owned(),
        );
        ns.insert(
            "xmlns".to_owned(),
            "http://www.w3.org/2000/xmlns/".to_owned(),
        );

        Parser {
            line: 1,
            col: 0,
            has_error: false,
            data: VecDeque::with_capacity(4096),
            buf: String::new(),
            namespaces: vec![ns],
            attributes: Vec::new(),
            st: State::OutsideTag,
            name: None,
            attr: None,
            delim: None,
            level: 0,
        }
    }

    /// Feeds a string slice to the parser
    pub fn feed_str(&mut self, data: &str) {
        self.data.extend(data.chars());
    }
}

impl Iterator for Parser {
    type Item = Result<Event, ParserError>;

    fn next(&mut self) -> Option<Result<Event, ParserError>> {
        if self.has_error {
            return None;
        }

        loop {
            let c = match self.data.pop_front() {
                Some(c) => c,
                None => return None,
            };

            if c == '\n' {
                self.line += 1;
                self.col = 0;
            } else {
                self.col += 1;
            }

            match self.parse_character(c) {
                Ok(None) => continue,
                Ok(Some(event)) => {
                    return Some(Ok(event));
                }
                Err(e) => {
                    self.has_error = true;
                    return Some(Err(e));
                }
            }
        }
    }
}

#[inline]
// Parse a QName to get Prefix and LocalPart
fn parse_qname(mut qname: String) -> (Option<String>, String) {
    if let Some(i) = qname.find(':') {
        let local = qname.split_off(i + 1);
        qname.pop();
        (Some(qname), local)
    } else {
        (None, qname)
    }
}

fn unescape_owned(input: String) -> Result<String, String> {
    if input.find('&').is_none() {
        Ok(input)
    } else {
        unescape(&input)
    }
}

impl Parser {
    // Get the namespace currently bound to a prefix.
    // Bindings are stored as a stack of HashMaps, we start searching in the top most HashMap
    // and traverse down until the prefix is found.
    fn namespace_for_prefix(&self, prefix: &str) -> Option<String> {
        for ns in self.namespaces.iter().rev() {
            if let Some(namespace) = ns.get(prefix) {
                if namespace.is_empty() {
                    return None;
                }
                return Some(namespace.clone());
            }
        }
        None
    }

    fn take_buf(&mut self) -> String {
        self.buf.split_off(0)
    }

    fn error(&self, msg: &'static str) -> Result<Option<Event>, ParserError> {
        Err(ParserError {
            line: self.line,
            col: self.col,
            msg,
        })
    }

    fn parse_character(&mut self, c: char) -> Result<Option<Event>, ParserError> {
        // println(fmt!("Now in state: %?", self.st));
        match self.st {
            State::OutsideTag => self.outside_tag(c),
            State::TagOpened => self.tag_opened(c),
            State::InProcessingInstructions => self.in_processing_instructions(c),
            State::InTagName => self.in_tag_name(c),
            State::InCloseTagName => self.in_close_tag_name(c),
            State::InTag => self.in_tag(c),
            State::InAttrName => self.in_attr_name(c),
            State::InAttrValue => self.in_attr_value(c),
            State::ExpectDelimiter => self.expect_delimiter(c),
            State::ExpectClose => self.expect_close(c),
            State::ExpectSpaceOrClose => self.expect_space_or_close(c),
            State::InExclamationMark => self.in_exclamation_mark(c),
            State::InCDATAOpening => self.in_cdata_opening(c),
            State::InCDATA => self.in_cdata(c),
            State::InCommentOpening => self.in_comment_opening(c),
            State::InComment1 => self.in_comment1(c),
            State::InComment2 => self.in_comment2(c),
            State::InDoctype => self.in_doctype(c),
        }
    }

    // Outside any tag, or other construct
    // '<' => TagOpened, producing Event::Characters
    fn outside_tag(&mut self, c: char) -> Result<Option<Event>, ParserError> {
        match c {
            '<' if self.buf.is_empty() => self.st = State::TagOpened,
            '<' => {
                self.st = State::TagOpened;
                let buf = match unescape_owned(self.take_buf()) {
                    Ok(unescaped) => unescaped,
                    Err(_) => return self.error("Found invalid entity"),
                };
                return Ok(Some(Event::Characters(buf)));
            }
            _ => self.buf.push(c),
        }
        Ok(None)
    }

    // Character following a '<', starting a tag or other construct
    // '?' => InProcessingInstructions
    // '!' => InExclamationMark
    // '/' => InCloseTagName
    //  _  => InTagName
    fn tag_opened(&mut self, c: char) -> Result<Option<Event>, ParserError> {
        self.st = match c {
            '?' => State::InProcessingInstructions,
            '!' => State::InExclamationMark,
            '/' => State::InCloseTagName,
            _ => {
                self.buf.push(c);
                State::InTagName
            }
        };
        Ok(None)
    }

    // Inside a processing instruction
    // '?' '>' => OutsideTag, producing PI
    fn in_processing_instructions(&mut self, c: char) -> Result<Option<Event>, ParserError> {
        match c {
            '?' => {
                self.level = 1;
                self.buf.push(c);
            }
            '>' if self.level == 1 => {
                self.level = 0;
                self.st = State::OutsideTag;
                let _ = self.buf.pop();
                let buf = self.take_buf();
                return Ok(Some(Event::PI(buf)));
            }
            _ => self.buf.push(c),
        }
        Ok(None)
    }

    // Inside a tag name (opening tag)
    // '/' => ExpectClose, producing Event::ElementStart
    // '>' => OutsideTag, producing Event::ElementStart
    // ' ' or '\t' or '\r' or '\n' => InTag
    fn in_tag_name(&mut self, c: char) -> Result<Option<Event>, ParserError> {
        match c {
            '/' | '>' => {
                let (prefix, name) = parse_qname(self.take_buf());
                let ns = match prefix {
                    None => self.namespace_for_prefix(""),
                    Some(ref pre) => match self.namespace_for_prefix(&pre) {
                        None => return self.error("Unbound namespace prefix in tag name"),
                        ns => ns,
                    },
                };

                self.namespaces.push(HashMap::new());
                self.st = if c == '/' {
                    self.name = Some((prefix.clone(), name.clone()));
                    State::ExpectClose
                } else {
                    State::OutsideTag
                };

                return Ok(Some(Event::ElementStart(StartTag {
                    name,
                    ns,
                    prefix,
                    attributes: HashMap::new(),
                })));
            }
            ' ' | '\t' | '\r' | '\n' => {
                self.namespaces.push(HashMap::new());
                self.name = Some(parse_qname(self.take_buf()));
                self.st = State::InTag;
            }
            _ => self.buf.push(c),
        }
        Ok(None)
    }

    // Inside a tag name (closing tag)
    // '>' => OutsideTag, producing ElementEnd
    // ' ' or '\t' or '\r' or '\n' => ExpectSpaceOrClose, producing ElementEnd
    fn in_close_tag_name(&mut self, c: char) -> Result<Option<Event>, ParserError> {
        match c {
            ' ' | '\t' | '\r' | '\n' | '>' => {
                let (prefix, name) = parse_qname(self.take_buf());

                let ns = match prefix {
                    None => self.namespace_for_prefix(""),
                    Some(ref pre) => match self.namespace_for_prefix(&pre) {
                        None => return self.error("Unbound namespace prefix in tag name"),
                        ns => ns,
                    },
                };

                self.namespaces.pop();
                self.st = if c == '>' {
                    State::OutsideTag
                } else {
                    State::ExpectSpaceOrClose
                };

                Ok(Some(Event::ElementEnd(EndTag { name, ns, prefix })))
            }
            _ => {
                self.buf.push(c);
                Ok(None)
            }
        }
    }

    // Inside a tag, parsing attributes
    // '/' => ExpectClose, producing StartTag
    // '>' => OutsideTag, producing StartTag
    // ' ' or '\t' or '\r' or '\n' => InAttrName
    fn in_tag(&mut self, c: char) -> Result<Option<Event>, ParserError> {
        match c {
            '/' | '>' => {
                let attributes = mem::replace(&mut self.attributes, Vec::new());
                let (prefix, name) = self
                    .name
                    .take()
                    .expect("Internal error: No element name set");
                let ns = match prefix {
                    None => self.namespace_for_prefix(""),
                    Some(ref pre) => match self.namespace_for_prefix(&pre) {
                        None => return self.error("Unbound namespace prefix in tag name"),
                        ns => ns,
                    },
                };

                let mut attributes_map: HashMap<(String, Option<String>), String> = HashMap::new();

                // At this point attribute namespaces are really just prefixes,
                // map them to the actual namespace
                for (name, ns, value) in attributes {
                    let ns = match ns {
                        None => None,
                        Some(ref prefix) => match self.namespace_for_prefix(&prefix) {
                            None => {
                                return self.error("Unbound namespace prefix in attribute name")
                            }
                            ns => ns,
                        },
                    };
                    if attributes_map.insert((name, ns), value).is_some() {
                        return self.error("Duplicate attribute");
                    }
                }

                self.st = if c == '/' {
                    self.name = Some((prefix.clone(), name.clone()));
                    State::ExpectClose
                } else {
                    State::OutsideTag
                };

                return Ok(Some(Event::ElementStart(StartTag {
                    name,
                    ns,
                    prefix,
                    attributes: attributes_map,
                })));
            }
            ' ' | '\t' | '\r' | '\n' => (),
            _ => {
                self.buf.push(c);
                self.st = State::InAttrName;
            }
        }
        Ok(None)
    }

    // Inside an attribute name
    // '=' => ExpectDelimiter
    fn in_attr_name(&mut self, c: char) -> Result<Option<Event>, ParserError> {
        match c {
            '=' => {
                self.level = 0;
                self.attr = Some(parse_qname(self.take_buf()));
                self.st = State::ExpectDelimiter;
            }
            ' ' | '\t' | '\r' | '\n' => self.level = 1,
            _ if self.level == 0 => self.buf.push(c),
            _ => return self.error("Space occured in attribute name"),
        }
        Ok(None)
    }

    // Inside an attribute value
    // delimiter => InTag, adds attribute
    fn in_attr_value(&mut self, c: char) -> Result<Option<Event>, ParserError> {
        if c == self
            .delim
            .expect("Internal error: In attribute value, but no delimiter set")
        {
            self.delim = None;
            self.st = State::InTag;
            let attr = self.attr.take();
            let (prefix, name) =
                attr.expect("Internal error: In attribute value, but no attribute name set");
            let value = match unescape_owned(self.take_buf()) {
                Ok(unescaped) => unescaped,
                Err(_) => return self.error("Found invalid entity"),
            };

            let last = self
                .namespaces
                .last_mut()
                .expect("Internal error: Empty namespace stack");
            match prefix {
                None if name == "xmlns" => {
                    last.insert(String::new(), value.clone());
                }
                Some(ref prefix) if prefix == "xmlns" => {
                    last.insert(name.clone(), value.clone());
                }
                _ => (),
            }

            self.attributes.push((name, prefix, value));
        } else {
            self.buf.push(c);
        }
        Ok(None)
    }

    // Looking for an attribute value delimiter
    // '"' or '\'' => InAttrValue, sets delimiter
    fn expect_delimiter(&mut self, c: char) -> Result<Option<Event>, ParserError> {
        match c {
            '"' | '\'' => {
                self.delim = Some(c);
                self.st = State::InAttrValue;
            }
            ' ' | '\t' | '\r' | '\n' => (),
            _ => return self.error("Attribute value not enclosed in ' or \""),
        }
        Ok(None)
    }

    // Expect closing '>' of an empty-element tag (no whitespace allowed)
    // '>' => OutsideTag
    fn expect_close(&mut self, c: char) -> Result<Option<Event>, ParserError> {
        match c {
            '>' => {
                self.st = State::OutsideTag;
                let (prefix, name) = self
                    .name
                    .take()
                    .expect("Internal error: No element name set");
                let ns = match prefix {
                    None => self.namespace_for_prefix(""),
                    Some(ref pre) => match self.namespace_for_prefix(&pre) {
                        None => return self.error("Unbound namespace prefix in tag name"),
                        ns => ns,
                    },
                };
                self.namespaces.pop();
                Ok(Some(Event::ElementEnd(EndTag { name, ns, prefix })))
            }
            _ => self.error("Expected '>' to close tag"),
        }
    }

    // Expect closing '>' of a start tag
    // '>' => OutsideTag
    fn expect_space_or_close(&mut self, c: char) -> Result<Option<Event>, ParserError> {
        match c {
            ' ' | '\t' | '\r' | '\n' => Ok(None),
            '>' => {
                self.st = State::OutsideTag;
                Ok(None)
            }
            _ => self.error("Expected '>' to close tag, or LWS"),
        }
    }

    // After an '!' trying to determine the type of the following construct
    // '-' => InCommentOpening
    // '[' => InCDATAOpening
    // 'D' => InDoctype
    fn in_exclamation_mark(&mut self, c: char) -> Result<Option<Event>, ParserError> {
        self.st = match c {
            '-' => State::InCommentOpening,
            '[' => State::InCDATAOpening,
            'D' => State::InDoctype,
            _ => return self.error("Malformed XML"),
        };
        Ok(None)
    }

    // Opening sequence of Event::CDATA
    // 'C' 'D' 'A' 'T' 'A' '[' => InCDATA
    fn in_cdata_opening(&mut self, c: char) -> Result<Option<Event>, ParserError> {
        static CDATA_PATTERN: [char; 6] = ['C', 'D', 'A', 'T', 'A', '['];
        if c == CDATA_PATTERN[self.level as usize] {
            self.level += 1;
        } else {
            return self.error("Invalid CDATA opening sequence");
        }

        if self.level == 6 {
            self.level = 0;
            self.st = State::InCDATA;
        }
        Ok(None)
    }

    // Inside CDATA
    // ']' ']' '>' => OutsideTag, producing Event::CDATA
    fn in_cdata(&mut self, c: char) -> Result<Option<Event>, ParserError> {
        match c {
            ']' => {
                self.buf.push(c);
                self.level += 1;
            }
            '>' if self.level >= 2 => {
                self.st = State::OutsideTag;
                self.level = 0;
                let len = self.buf.len();
                self.buf.truncate(len - 2);
                let buf = self.take_buf();
                return Ok(Some(Event::CDATA(buf)));
            }
            _ => {
                self.buf.push(c);
                self.level = 0;
            }
        }
        Ok(None)
    }

    // Opening sequence of a comment
    // '-' => InComment1
    fn in_comment_opening(&mut self, c: char) -> Result<Option<Event>, ParserError> {
        if c == '-' {
            self.st = State::InComment1;
            self.level = 0;
            Ok(None)
        } else {
            self.error("Expected 2nd '-' to start comment")
        }
    }

    // Inside a comment
    // '-' '-' => InComment2
    fn in_comment1(&mut self, c: char) -> Result<Option<Event>, ParserError> {
        if c == '-' {
            self.level += 1;
        } else {
            self.level = 0;
        }

        if self.level == 2 {
            self.level = 0;
            self.st = State::InComment2;
        }

        self.buf.push(c);

        Ok(None)
    }

    // Closing a comment
    // '>' => OutsideTag, producing Comment
    fn in_comment2(&mut self, c: char) -> Result<Option<Event>, ParserError> {
        if c != '>' {
            self.error("No more than one adjacent '-' allowed in a comment")
        } else {
            self.st = State::OutsideTag;
            let len = self.buf.len();
            self.buf.truncate(len - 2);
            let buf = self.take_buf();
            Ok(Some(Event::Comment(buf)))
        }
    }

    // Inside a doctype
    // '>' after appropriate opening => OutsideTag
    fn in_doctype(&mut self, c: char) -> Result<Option<Event>, ParserError> {
        static DOCTYPE_PATTERN: [char; 6] = ['O', 'C', 'T', 'Y', 'P', 'E'];
        match self.level {
            0..=5 => {
                if c == DOCTYPE_PATTERN[self.level as usize] {
                    self.level += 1;
                } else {
                    return self.error("Invalid DOCTYPE");
                }
            }
            6 => {
                match c {
                    ' ' | '\t' | '\r' | '\n' => (),
                    _ => return self.error("Invalid DOCTYPE"),
                }
                self.level += 1;
            }
            _ if c == '>' => {
                self.level = 0;
                self.st = State::OutsideTag;
            }
            _ => (),
        }
        Ok(None)
    }
}

#[cfg(test)]
mod parser_tests {
    use std::collections::HashMap;

    use super::super::{EndTag, Event, ParserError, StartTag};
    use super::Parser;

    #[test]
    fn test_start_tag() {
        let mut p = Parser::new();
        let mut i = 0u8;
        p.feed_str("<a>");
        for event in p {
            i += 1;
            assert_eq!(
                event,
                Ok(Event::ElementStart(StartTag {
                    name: "a".to_owned(),
                    ns: None,
                    prefix: None,
                    attributes: HashMap::new()
                })),
            );
        }
        assert_eq!(i, 1u8);
    }

    #[test]
    fn test_end_tag() {
        let mut p = Parser::new();
        let mut i = 0u8;
        p.feed_str("</a>");
        for event in p {
            i += 1;
            assert_eq!(
                event,
                Ok(Event::ElementEnd(EndTag {
                    name: "a".to_owned(),
                    ns: None,
                    prefix: None
                })),
            );
        }
        assert_eq!(i, 1u8);
    }

    #[test]
    fn test_self_closing_with_space() {
        let mut p = Parser::new();
        p.feed_str("<register />");

        let v: Vec<Result<Event, ParserError>> = p.collect();
        assert_eq!(
            v,
            vec![
                Ok(Event::ElementStart(StartTag {
                    name: "register".to_owned(),
                    ns: None,
                    prefix: None,
                    attributes: HashMap::new()
                })),
                Ok(Event::ElementEnd(EndTag {
                    name: "register".to_owned(),
                    ns: None,
                    prefix: None,
                }))
            ],
        );
    }

    #[test]
    fn test_self_closing_without_space() {
        let mut p = Parser::new();
        p.feed_str("<register/>");

        let v: Vec<Result<Event, ParserError>> = p.collect();
        assert_eq!(
            v,
            vec![
                Ok(Event::ElementStart(StartTag {
                    name: "register".to_owned(),
                    ns: None,
                    prefix: None,
                    attributes: HashMap::new()
                })),
                Ok(Event::ElementEnd(EndTag {
                    name: "register".to_owned(),
                    ns: None,
                    prefix: None,
                }))
            ],
        );
    }

    #[test]
    fn test_self_closing_namespace() {
        let mut p = Parser::new();
        p.feed_str("<foo:a xmlns:foo='urn:foo'/>");

        let v: Vec<Result<Event, ParserError>> = p.collect();
        let mut attr: HashMap<(String, Option<String>), String> = HashMap::new();
        attr.insert(
            (
                "foo".to_owned(),
                Some("http://www.w3.org/2000/xmlns/".to_owned()),
            ),
            "urn:foo".to_owned(),
        );
        assert_eq!(
            v,
            vec![
                Ok(Event::ElementStart(StartTag {
                    name: "a".to_owned(),
                    ns: Some("urn:foo".to_owned()),
                    prefix: Some("foo".to_owned()),
                    attributes: attr,
                })),
                Ok(Event::ElementEnd(EndTag {
                    name: "a".to_owned(),
                    ns: Some("urn:foo".to_owned()),
                    prefix: Some("foo".to_owned()),
                }))
            ],
        );
    }

    #[test]
    fn test_pi() {
        let mut p = Parser::new();
        let mut i = 0u8;
        p.feed_str("<?xml version='1.0' encoding='utf-8'?>");
        for event in p {
            i += 1;
            assert_eq!(
                event,
                Ok(Event::PI("xml version='1.0' encoding='utf-8'".to_owned())),
            );
        }
        assert_eq!(i, 1u8);
    }

    #[test]
    fn test_comment() {
        let mut p = Parser::new();
        let mut i = 0u8;
        p.feed_str("<!--Nothing to see-->");
        for event in p {
            i += 1;
            assert_eq!(event, Ok(Event::Comment("Nothing to see".to_owned())));
        }
        assert_eq!(i, 1u8);
    }
    #[test]
    fn test_cdata() {
        let mut p = Parser::new();
        let mut i = 0u8;
        p.feed_str("<![CDATA[<html><head><title>x</title></head><body/></html>]]>");
        for event in p {
            i += 1;
            assert_eq!(
                event,
                Ok(Event::CDATA(
                    "<html><head><title>x</title></head><body/></html>".to_owned()
                )),
            );
        }
        assert_eq!(i, 1u8);
    }

    #[test]
    fn test_characters() {
        let mut p = Parser::new();
        let mut i = 0u8;
        p.feed_str("<text>Hello World, it&apos;s a nice day</text>");
        for event in p {
            i += 1;
            if i == 2 {
                assert_eq!(
                    event,
                    Ok(Event::Characters("Hello World, it's a nice day".to_owned())),
                );
            }
        }
        assert_eq!(i, 3u8);
    }

    #[test]
    fn test_doctype() {
        let mut p = Parser::new();
        let mut i = 0u8;
        p.feed_str("<!DOCTYPE html>");
        for _ in p {
            i += 1;
        }
        assert_eq!(i, 0u8);
    }
}
