// RustyXML
// Copyright 2013-2016 RustyXML developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![crate_name = "xml"]
#![crate_type = "lib"]
#![forbid(non_camel_case_types)]
#![warn(missing_docs)]
// Required for benchmarks
#![cfg_attr(feature = "bench", feature(test))]

/*!
 * An XML parsing library
 */

pub use crate::element::ChildElements;
pub use crate::element::Element;
pub use crate::element_builder::BuilderError;
pub use crate::element_builder::ElementBuilder;
pub use crate::parser::Event;
pub use crate::parser::Parser;
pub use crate::parser::ParserError;

use std::char;
use std::collections::HashMap;
use std::fmt;

mod element;
mod element_builder;
mod parser;

// General functions

#[inline]
/// Escapes ', ", &, <, and > with the appropriate XML entities.
pub fn escape(input: &str) -> String {
    let mut result = String::with_capacity(input.len());

    for c in input.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '\'' => result.push_str("&apos;"),
            '"' => result.push_str("&quot;"),
            o => result.push(o),
        }
    }
    result
}

#[inline]
/// Unescapes all valid XML entities in a string.
/// Returns the first invalid entity on failure.
pub fn unescape(input: &str) -> Result<String, String> {
    let mut result = String::with_capacity(input.len());

    let mut it = input.split('&');

    // Push everything before the first '&'
    if let Some(sub) = it.next() {
        result.push_str(sub);
    }

    for sub in it {
        match sub.find(';') {
            Some(idx) => {
                let ent = &sub[..idx];
                match ent {
                    "quot" => result.push('"'),
                    "apos" => result.push('\''),
                    "gt" => result.push('>'),
                    "lt" => result.push('<'),
                    "amp" => result.push('&'),
                    ent => {
                        let val = if ent.starts_with("#x") {
                            u32::from_str_radix(&ent[2..], 16).ok()
                        } else if ent.starts_with('#') {
                            u32::from_str_radix(&ent[1..], 10).ok()
                        } else {
                            None
                        };
                        match val.and_then(char::from_u32) {
                            Some(c) => result.push(c),
                            None => return Err(format!("&{};", ent)),
                        }
                    }
                }
                result.push_str(&sub[idx + 1..]);
            }
            None => return Err("&".to_owned() + sub),
        }
    }
    Ok(result)
}

// General types
#[derive(Clone, PartialEq, Debug)]
/// An Enum describing a XML Node
pub enum Xml {
    /// An XML Element
    ElementNode(Element),
    /// Character Data
    CharacterNode(String),
    /// CDATA
    CDATANode(String),
    /// A XML Comment
    CommentNode(String),
    /// Processing Information
    PINode(String),
}

#[derive(PartialEq, Eq, Debug)]
/// Structure describing an opening tag
pub struct StartTag {
    /// The tag's name
    pub name: String,
    /// The tag's namespace
    pub ns: Option<String>,
    /// The tag's prefix
    pub prefix: Option<String>,
    /// The tag's attributes
    pub attributes: HashMap<(String, Option<String>), String>,
}

#[derive(PartialEq, Eq, Debug)]
/// Structure describing a closing tag
pub struct EndTag {
    /// The tag's name
    pub name: String,
    /// The tag's namespace
    pub ns: Option<String>,
    /// The tag's prefix
    pub prefix: Option<String>,
}

impl fmt::Display for Xml {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Xml::ElementNode(ref elem) => elem.fmt(f),
            Xml::CharacterNode(ref data) => write!(f, "{}", escape(&data)),
            Xml::CDATANode(ref data) => write!(f, "<![CDATA[{}]]>", &data),
            Xml::CommentNode(ref data) => write!(f, "<!--{}-->", &data),
            Xml::PINode(ref data) => write!(f, "<?{}?>", &data),
        }
    }
}

#[cfg(test)]
mod lib_tests {
    use super::{escape, unescape, Element, Xml};

    #[test]
    fn test_escape() {
        let esc = escape("&<>'\"");
        assert_eq!(esc, "&amp;&lt;&gt;&apos;&quot;");
    }

    #[test]
    fn test_unescape() {
        let unesc = unescape("&amp;lt;&lt;&gt;&apos;&quot;&#x201c;&#x201d;&#38;&#34;");
        assert_eq!(
            unesc.as_ref().map(|x| &x[..]),
            Ok("&lt;<>'\"\u{201c}\u{201d}&\""),
        );
    }

    #[test]
    fn test_unescape_invalid() {
        let unesc = unescape("&amp;&nbsp;");
        assert_eq!(unesc.as_ref().map_err(|x| &x[..]), Err("&nbsp;"));
    }

    #[test]
    fn test_show_element() {
        let elem = Element::new("a".to_owned(), None, vec![]);
        assert_eq!(format!("{}", elem), "<a/>");

        let elem = Element::new(
            "a".to_owned(),
            None,
            vec![("href".to_owned(), None, "http://rust-lang.org".to_owned())],
        );
        assert_eq!(format!("{}", elem), "<a href='http://rust-lang.org'/>");

        let mut elem = Element::new("a".to_owned(), None, vec![]);
        elem.tag(Element::new("b".to_owned(), None, vec![]));
        assert_eq!(format!("{}", elem), "<a><b/></a>");

        let mut elem = Element::new(
            "a".to_owned(),
            None,
            vec![("href".to_owned(), None, "http://rust-lang.org".to_owned())],
        );
        elem.tag(Element::new("b".to_owned(), None, vec![]));
        assert_eq!(
            format!("{}", elem),
            "<a href='http://rust-lang.org'><b/></a>",
        );
    }

    #[test]
    fn test_show_element_xmlns() {
        let elem: Element = "<a xmlns='urn:test'/>".parse().unwrap();
        assert_eq!(format!("{}", elem), "<a xmlns='urn:test'/>");

        let elem: Element = "<a xmlns='urn:test'><b xmlns='urn:toast'/></a>"
            .parse()
            .unwrap();
        assert_eq!(
            format!("{}", elem),
            "<a xmlns='urn:test'><b xmlns='urn:toast'/></a>",
        );

        let elem = Element::new(
            "a".to_owned(),
            Some("urn:test".to_owned()),
            vec![("href".to_owned(), None, "http://rust-lang.org".to_owned())],
        );
        assert_eq!(
            format!("{}", elem),
            "<a xmlns='urn:test' href='http://rust-lang.org'/>",
        );
    }

    #[test]
    fn test_show_characters() {
        let chars = Xml::CharacterNode("some text".to_owned());
        assert_eq!(format!("{}", chars), "some text");
    }

    #[test]
    fn test_show_cdata() {
        let chars = Xml::CDATANode("some text".to_owned());
        assert_eq!(format!("{}", chars), "<![CDATA[some text]]>");
    }

    #[test]
    fn test_show_comment() {
        let chars = Xml::CommentNode("some text".to_owned());
        assert_eq!(format!("{}", chars), "<!--some text-->");
    }

    #[test]
    fn test_show_pi() {
        let chars = Xml::PINode("xml version='1.0'".to_owned());
        assert_eq!(format!("{}", chars), "<?xml version='1.0'?>");
    }

    #[test]
    fn test_content_str() {
        let mut elem = Element::new("a".to_owned(), None, vec![]);
        elem.pi("processing information".to_owned())
            .cdata("<hello/>".to_owned())
            .tag_stay(Element::new("b".to_owned(), None, vec![]))
            .text("World".to_owned())
            .comment("Nothing to see".to_owned());
        assert_eq!(elem.content_str(), "<hello/>World");
    }
}

#[cfg(test)]
#[cfg(feature = "bench")]
mod lib_bench {
    extern crate test;

    use self::test::Bencher;
    use super::{escape, unescape};
    use std::iter::repeat;

    #[bench]
    fn bench_escape(bh: &mut Bencher) {
        let input: String = repeat("&<>'\"").take(100).collect();
        bh.iter(|| escape(&input));
        bh.bytes = input.len() as u64;
    }

    #[bench]
    fn bench_unescape(bh: &mut Bencher) {
        let input: String = repeat("&amp;&lt;&gt;&apos;&quot;").take(50).collect();
        bh.iter(|| unescape(&input));
        bh.bytes = input.len() as u64;
    }
}
