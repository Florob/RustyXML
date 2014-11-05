// RustyXML
// Copyright (c) 2013, 2014 Florian Zeitz
//
// This project is MIT licensed.
// Please see the COPYING file for more information.
//
// The parser herein is derived from OFXMLParser as included with
// ObjFW, Copyright (c) 2008-2013 Jonathan Schleifer.
// Permission to license this derived work under MIT license has been granted by ObjFW's author.

use super::{Event, PI, ElementStart, ElementEnd, Characters, CDATA, Comment,
            StartTag, EndTag, unescape};
use std::collections::{HashMap, RingBuf};
use std::mem;
use std::iter::Iterator;

#[deriving(PartialEq, Show)]
/// The structure returned, when erroneous XML is read
pub struct Error {
    /// The line number at which the error occurred
    pub line: uint,
    /// The column number at which the error occurred
    pub col: uint,
    /// A message describing the type of the error
    pub msg: &'static str
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

/// A streaming XML parser
pub struct Parser {
    line: uint,
    col: uint,
    has_error: bool,
    data: RingBuf<char>,
    buf: String,
    namespaces: Vec<HashMap<String, String>>,
    attributes: Vec<(String, Option<String>, String)>,
    st: State,
    name: Option<(Option<String>, String)>,
    attr: Option<(Option<String>, String)>,
    delim: Option<char>,
    level: uint
}

impl Parser {
    /// Returns a new `Parser`
    pub fn new() -> Parser {
        let mut ns = HashMap::with_capacity(2);
        // Add standard namespaces
        ns.swap("xml".to_string(), "http://www.w3.org/XML/1998/namespace".to_string());
        ns.swap("xmlns".to_string(), "http://www.w3.org/2000/xmlns/".to_string());

        Parser {
            line: 1,
            col: 0,
            has_error: false,
            data: RingBuf::with_capacity(4096),
            buf: String::new(),
            namespaces: vec![ns],
            attributes: Vec::new(),
            st: OutsideTag,
            name: None,
            attr: None,
            delim: None,
            level: 0
        }
    }

    /**
     * Feeds the string `data` to the parser.
     * The `Event`s, and `Error`s generated while parsing the string
     * can be requested by iterating over the parser
     *
     * ~~~
     * use xml::Parser;
     *
     * let mut p = Parser::new();
     * p.feed_str("<a href='http://rust-lang.org'>Rust</a>");
     * for event in p {
     *     match event {
     *        // [...]
     *        _ => ()
     *     }
     * }
     * ~~~
     */
    pub fn feed_str(&mut self, data: &str) {
        let iter = data.chars();
        let len = self.data.len() + iter.size_hint().val0();
        self.data.reserve(len);
        self.data.extend(iter);
    }
}

impl Iterator<Result<Event, Error>> for Parser {
    fn next(&mut self) -> Option<Result<Event, Error>> {
        if self.has_error {
            return None;
        }

        loop {
            let c = match self.data.pop_front() {
                Some(c) => c,
                None => return None
            };

            if c == '\n' {
                self.line += 1u;
                self.col = 0u;
            } else {
                self.col += 1u;
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
fn parse_qname(qname: &str) -> (Option<String>, String) {
    if let Some(i) = qname.find(':') {
        (Some(qname[..i].to_string()), qname[i+1..].to_string())
    } else {
        (None, qname.to_string())
    }
}

impl Parser {
    // Get the namespace currently bound to a prefix.
    // Bindings are stored as a stack of HashMaps, we start searching in the top most HashMap
    // and traverse down until the prefix is found.
    fn namespace_for_prefix(&self, prefix: &str) -> Option<String> {
        for ns in self.namespaces[].iter().rev() {
            if let Some(namespace) = ns.find_equiv(prefix) {
                if namespace.len() == 0 {
                    return None;
                }
                return Some(namespace.clone());
            }
        }
        None
    }

    fn error(&self, msg: &'static str) -> Result<Option<Event>, Error> {
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
            InCDATAOpening => self.in_cdata_opening(c),
            InCDATA => self.in_cdata(c),
            InCommentOpening => self.in_comment_opening(c),
            InComment1 => self.in_comment1(c),
            InComment2 => self.in_comment2(c),
            InDoctype => self.in_doctype(c),
        }
    }

    // Outside any tag, or other construct
    // '<' => TagOpened, producing Characters
    fn outside_tag(&mut self, c: char) -> Result<Option<Event>, Error> {
        match c {
            '<' if self.buf.len() > 0 => {
                self.st = TagOpened;
                let buf = match unescape(self.buf[]) {
                    Ok(unescaped) => unescaped,
                    Err(_) => return self.error("Found invalid entity")
                };
                self.buf.truncate(0);
                return Ok(Some(Characters(buf)));
            }
            '<' => self.st = TagOpened,
            _ => self.buf.push(c)
        }
        Ok(None)
    }

    // Character following a '<', starting a tag or other construct
    // '?' => InProcessingInstructions
    // '!' => InExclamationMark
    // '/' => InCloseTagName
    //  _  => InTagName
    fn tag_opened(&mut self, c: char) -> Result<Option<Event>, Error> {
        self.st = match c {
            '?' => InProcessingInstructions,
            '!' => InExclamationMark,
            '/' => InCloseTagName,
            _ => {
                self.buf.push(c);
                InTagName
            }
        };
        Ok(None)
    }

    // Inside a processing instruction
    // '?' '>' => OutsideTag, producing PI
    fn in_processing_instructions(&mut self, c: char) -> Result<Option<Event>, Error> {
        match c {
            '?' => {
                self.level = 1;
                self.buf.push(c);
            }
            '>' if self.level == 1 => {
                self.level = 0;
                self.st = OutsideTag;
                let _ = self.buf.pop();
                let buf = mem::replace(&mut self.buf, String::new());
                return Ok(Some(PI(buf)));
            }
            _ => self.buf.push(c)
        }
        Ok(None)
    }

    // Inside a tag name (opening tag)
    // '/' => ExpectClose, producing ElementStart
    // '>' => OutsideTag, producing ElementStart
    // ' ' or '\t' or '\r' or '\n' => InTag
    fn in_tag_name(&mut self, c: char) -> Result<Option<Event>, Error> {
        match c {
            '/'
            | '>' => {
                let (prefix, name) = parse_qname(self.buf[]);
                self.buf.truncate(0);
                let ns = match prefix {
                    None => self.namespace_for_prefix(""),
                    Some(ref pre) => match self.namespace_for_prefix(pre[]) {
                        None => return self.error("Unbound namespace prefix in tag name"),
                        ns => ns
                    }
                };

                self.namespaces.push(HashMap::new());
                self.st = if c == '/' {
                    self.name = Some((prefix.clone(), name.clone()));
                    ExpectClose
                } else {
                    OutsideTag
                };

                return Ok(Some(ElementStart(StartTag {
                    name: name,
                    ns: ns,
                    prefix: prefix,
                    attributes: HashMap::new()
                })));
            }
            ' '
            | '\t'
            | '\r'
            | '\n' => {
                self.namespaces.push(HashMap::new());
                self.name = Some(parse_qname(self.buf[]));
                self.buf.truncate(0);
                self.st = InTag;
            }
            _ => self.buf.push(c)
        }
        Ok(None)
    }

    // Inside a tag name (closing tag)
    // '>' => OutsideTag, producing EndTag
    // ' ' or '\t' or '\r' or '\n' => ExpectSpaceOrClose, producing EndTag
    fn in_close_tag_name(&mut self, c: char) -> Result<Option<Event>, Error> {
        match c {
            ' '
            | '\t'
            | '\r'
            | '\n'
            | '>' => {
                let (prefix, name) = parse_qname(self.buf[]);
                self.buf.truncate(0);

                let ns = match prefix {
                    None => self.namespace_for_prefix(""),
                    Some(ref pre) => match self.namespace_for_prefix(pre[]) {
                        None => return self.error("Unbound namespace prefix in tag name"),
                        ns => ns
                    }
                };

                self.namespaces.pop();
                self.st = if c == '>' {
                    OutsideTag
                } else {
                    ExpectSpaceOrClose
                };

                Ok(Some(ElementEnd(EndTag { name: name, ns: ns, prefix: prefix })))
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
    fn in_tag(&mut self, c: char) -> Result<Option<Event>, Error> {
        match c {
            '/'
            | '>' => {
                let attributes = mem::replace(&mut self.attributes, Vec::new());
                let (prefix, name) = self.name.take().expect("Internal error: No element name set");
                let ns = match prefix {
                    None => self.namespace_for_prefix(""),
                    Some(ref pre) => match self.namespace_for_prefix(pre[]) {
                        None => return self.error("Unbound namespace prefix in tag name"),
                        ns => ns
                    }
                };

                let mut attributes_map: HashMap<(String, Option<String>), String> = HashMap::new();

                // At this point attribute namespaces are really just prefixes,
                // map them to the actual namespace
                for (name, ns, value) in attributes.into_iter() {
                    let ns = match ns {
                        None => None,
                        Some(ref prefix) => match self.namespace_for_prefix(prefix[]) {
                            None => return self.error("Unbound namespace prefix in attribute name"),
                            ns => ns
                        }
                    };
                    if !attributes_map.insert((name, ns), value) {
                        return self.error("Duplicate attribute");
                    }
                }

                self.st = if c == '/' {
                    self.name = Some((prefix.clone(), name.clone()));
                    ExpectClose
                } else {
                    OutsideTag
                };

                return Ok(Some(ElementStart(StartTag {
                    name: name,
                    ns: ns,
                    prefix: prefix,
                    attributes: attributes_map
                })));
            }
            ' '
            | '\t'
            | '\r'
            | '\n' => (),
            _ => {
                self.buf.push(c);
                self.st = InAttrName;
            }
        }
        Ok(None)
    }

    // Inside an attribute name
    // '=' => ExpectDelimiter
    fn in_attr_name(&mut self, c: char) -> Result<Option<Event>, Error> {
        match c {
            '=' => {
                self.level = 0;
                self.attr = Some(parse_qname(self.buf[]));
                self.buf.truncate(0);
                self.st = ExpectDelimiter;
            }
            ' '
            | '\t'
            | '\r'
            | '\n' => self.level = 1,
            _ if self.level == 0 => self.buf.push(c),
            _ => return self.error("Space occured in attribute name")
        }
        Ok(None)
    }

    // Inside an attribute value
    // delimiter => InTag, adds attribute
    fn in_attr_value(&mut self, c: char) -> Result<Option<Event>, Error> {
        if c == self.delim.expect("Internal error: In attribute value, but no delimiter set") {
            self.delim = None;
            self.st = InTag;
            let attr = self.attr.take();
            let (prefix, name) =
                attr.expect("Internal error: In attribute value, but no attribute name set");
            let value = match unescape(self.buf[]) {
                Ok(unescaped) => unescaped,
                Err(_) => return self.error("Found invalid entity")
            };
            self.buf.truncate(0);

            let last = self.namespaces.last_mut().expect("Internal error: Empty namespace stack");
            match prefix {
                None if name[] == "xmlns" => {
                    last.swap(String::new(), value.clone());
                }
                Some(ref prefix) if prefix[] == "xmlns" => {
                    last.swap(name.clone(), value.clone());
                }
                _ => ()
            }

            self.attributes.push((name, prefix, value));
        } else {
            self.buf.push(c);
        }
        Ok(None)
    }

    // Looking for an attribute value delimiter
    // '"' or '\'' => InAttrValue, sets delimiter
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
            _ => return self.error("Attribute value not enclosed in ' or \"")
        }
        Ok(None)
    }

    // Expect closing '>' of an empty-element tag (no whitespace allowed)
    // '>' => OutsideTag
    fn expect_close(&mut self, c: char) -> Result<Option<Event>, Error> {
        match c {
            '>' => {
                self.st = OutsideTag;
                let (prefix, name) = self.name.take().expect("Internal error: No element name set");
                let ns = match prefix {
                    None => self.namespace_for_prefix(""),
                    Some(ref pre) => match self.namespace_for_prefix(pre[]) {
                        None => return self.error("Unbound namespace prefix in tag name"),
                        ns => ns
                    }
                };
                self.namespaces.pop();
                Ok(Some(ElementEnd(EndTag { name: name, ns: ns, prefix: prefix })))
            }
            _ => self.error("Expected '>' to close tag")
       }
    }

    // Expect closing '>' of a start tag
    // '>' => OutsideTag
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
            _ => self.error("Expected '>' to close tag, or LWS")
       }
    }

    // After an '!' trying to determine the type of the following construct
    // '-' => InCommentOpening
    // '[' => InCDATAOpening
    // 'D' => InDoctype
    fn in_exclamation_mark(&mut self, c: char) -> Result<Option<Event>, Error> {
        self.st = match c {
            '-' => InCommentOpening,
            '[' => InCDATAOpening,
            'D' => InDoctype,
            _ => return self.error("Malformed XML")
        };
        Ok(None)
    }

    // Opening sequence of CDATA
    // 'C' 'D' 'A' 'T' 'A' '[' => InCDATA
    fn in_cdata_opening(&mut self, c: char) -> Result<Option<Event>, Error> {
        static CDATA_PATTERN: [char, ..6] = ['C', 'D', 'A', 'T', 'A', '['];
        if c == CDATA_PATTERN[self.level] {
            self.level += 1;
        } else {
            return self.error("Invalid CDATA opening sequence")
        }

        if self.level == 6 {
            self.level = 0;
            self.st = InCDATA;
        }
        Ok(None)
    }

    // Inside CDATA
    // ']' ']' '>' => OutsideTag, producing CDATA
    fn in_cdata(&mut self, c: char) -> Result<Option<Event>, Error> {
        match c {
            ']' => {
                self.buf.push(c);
                self.level += 1;
            }
            '>' if self.level >= 2 => {
                self.st = OutsideTag;
                self.level = 0;
                let len = self.buf.len();
                self.buf.truncate(len - 2);
                let buf = mem::replace(&mut self.buf, String::new());
                return Ok(Some(CDATA(buf)))
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
    fn in_comment_opening(&mut self, c: char) -> Result<Option<Event>, Error> {
        if c == '-' {
            self.st = InComment1;
            self.level = 0;
            Ok(None)
        } else {
            self.error("Expected 2nd '-' to start comment")
        }
    }

    // Inside a comment
    // '-' '-' => InComment2
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

        self.buf.push(c);

        Ok(None)
    }

    // Closing a comment
    // '>' => OutsideTag, producing Comment
    fn in_comment2(&mut self, c: char) -> Result<Option<Event>, Error> {
        if c != '>' {
            self.error("No more than one adjacent '-' allowed in a comment")
        } else {
            self.st = OutsideTag;
            let len = self.buf.len();
            self.buf.truncate(len - 2);
            let buf = mem::replace(&mut self.buf, String::new());
            Ok(Some(Comment(buf)))
        }
    }

    // Inside a doctype
    // '>' after appropriate opening => OutsideTag
    fn in_doctype(&mut self, c: char) -> Result<Option<Event>, Error> {
        static DOCTYPE_PATTERN: [char, ..6] = ['O', 'C', 'T', 'Y', 'P', 'E'];
        match self.level {
            0...5 => if c == DOCTYPE_PATTERN[self.level] {
                self.level += 1;
            } else {
                return self.error("Invalid DOCTYPE");
            },
            6 => {
                match c {
                    ' '
                    | '\t'
                    | '\r'
                    | '\n' => (),
                    _ => return self.error("Invalid DOCTYPE")
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

#[cfg(test)]
mod parser_tests {
    use std::collections::HashMap;

    use super::Parser;
    use super::super::{Event, Error, PI, ElementStart, ElementEnd, Comment, CDATA, Characters,
                       StartTag, EndTag};

    #[test]
    fn test_start_tag() {
        let mut p = Parser::new();
        let mut i = 0;
        p.feed_str("<a>");
        for event in p {
            i += 1;
            assert_eq!(event, Ok(ElementStart(StartTag {
                name: "a".to_string(),
                ns: None,
                prefix: None,
                attributes: HashMap::new()
            })));
        }
        assert_eq!(i, 1u);
    }

    #[test]
    fn test_end_tag() {
        let mut p = Parser::new();
        let mut i = 0;
        p.feed_str("</a>");
        for event in p {
            i += 1;
            assert_eq!(event, Ok(ElementEnd(EndTag {
                name: "a".to_string(),
                ns: None,
                prefix: None
            })));
        }
        assert_eq!(i, 1u);
    }

    #[test]
    fn test_self_closing_with_space() {
        let mut p = Parser::new();
        p.feed_str("<register />");

        let v: Vec<Result<Event, Error>> = p.collect();
        assert_eq!(v, vec![
            Ok(ElementStart(StartTag {
                name: "register".to_string(),
                ns: None,
                prefix: None,
                attributes: HashMap::new()
            })),
            Ok(ElementEnd(EndTag {
                name: "register".to_string(),
                ns: None,
                prefix: None,
            }))
        ]);
    }

    #[test]
    fn test_self_closing_without_space() {
        let mut p = Parser::new();
        p.feed_str("<register/>");

        let v: Vec<Result<Event, Error>> = p.collect();
        assert_eq!(v, vec![
            Ok(ElementStart(StartTag {
                name: "register".to_string(),
                ns: None,
                prefix: None,
                attributes: HashMap::new()
            })),
            Ok(ElementEnd(EndTag {
                name: "register".to_string(),
                ns: None,
                prefix: None,
            }))
        ]);
    }

    #[test]
    fn test_self_closing_namespace() {
        let mut p = Parser::new();
        p.feed_str("<foo:a xmlns:foo='urn:foo'/>");

        let v: Vec<Result<Event, Error>> = p.collect();
        let mut attr: HashMap<(String, Option<String>), String> = HashMap::new();
        attr.insert(("foo".to_string(), Some("http://www.w3.org/2000/xmlns/".to_string())),
                    "urn:foo".to_string());
        assert_eq!(v, vec![
            Ok(ElementStart(StartTag {
                name: "a".to_string(),
                ns: Some("urn:foo".to_string()),
                prefix: Some("foo".to_string()),
                attributes: attr,
            })),
            Ok(ElementEnd(EndTag {
                name: "a".to_string(),
                ns: Some("urn:foo".to_string()),
                prefix: Some("foo".to_string()),
            }))
        ]);
    }

    #[test]
    fn test_pi() {
        let mut p = Parser::new();
        let mut i = 0;
        p.feed_str("<?xml version='1.0' encoding='utf-8'?>");
        for event in p {
            i += 1;
            assert_eq!(event, Ok(PI("xml version='1.0' encoding='utf-8'".to_string())));
        }
        assert_eq!(i, 1u);
    }

    #[test]
    fn test_comment() {
        let mut p = Parser::new();
        let mut i = 0;
        p.feed_str("<!--Nothing to see-->");
        for event in p {
            i += 1;
            assert_eq!(event, Ok(Comment("Nothing to see".to_string())));
        }
        assert_eq!(i, 1u);
    }
    #[test]
    fn test_cdata() {
        let mut p = Parser::new();
        let mut i = 0;
        p.feed_str("<![CDATA[<html><head><title>x</title></head><body/></html>]]>");
        for event in p {
            i += 1;
            assert_eq!(event,
                       Ok(CDATA("<html><head><title>x</title></head><body/></html>".to_string())));
        }
        assert_eq!(i, 1u);
    }

    #[test]
    fn test_characters() {
        let mut p = Parser::new();
        let mut i = 0;
        p.feed_str("<text>Hello World, it&apos;s a nice day</text>");
        for event in p {
            i += 1;
            if i == 2 {
                assert_eq!(event, Ok(Characters("Hello World, it's a nice day".to_string())));
            }
        }
        assert_eq!(i, 3u);
    }

    #[test]
    fn test_doctype() {
        let mut p = Parser::new();
        let mut i = 0;
        p.feed_str("<!DOCTYPE html>");
        for _ in p {
            i += 1;
        }
        assert_eq!(i, 0u);
    }
}
