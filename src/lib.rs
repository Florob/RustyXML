// RustyXML
// Copyright (c) 2013-2015 Florian Zeitz
//
// This project is MIT licensed.
// Please see the COPYING file for more information.

#![crate_name = "xml"]
#![crate_type = "lib" ]
#![forbid(non_camel_case_types)]
#![warn(missing_docs)]

// These are unstable for now
#![cfg_attr(test, feature(test))]

/*!
 * An XML parsing library
 */

pub use parser::Event;
pub use parser::Parser;
pub use parser::ParserError;
pub use element::ChildElements;
pub use element::Element;
pub use element_builder::ElementBuilder;
pub use element_builder::BuilderError;

use std::char;
use std::fmt;
use std::collections::HashMap;

mod parser;
mod element;
mod element_builder;

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
            o => result.push(o)
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
                    "gt"   => result.push('>'),
                    "lt"   => result.push('<'),
                    "amp"  => result.push('&'),
                    ent => {
                        let val = if ent.starts_with("#x") {
                            u32::from_str_radix(&ent[2..], 16).ok()
                        } else if ent.starts_with("#") {
                            u32::from_str_radix(&ent[1..], 10).ok()
                        } else {
                            None
                        };
                        match val.and_then(char::from_u32) {
                            Some(c) => result.push(c),
                            None => return Err(format!("&{};", ent))
                        }
                    }
                }
                result.push_str(&sub[idx+1..]);
            }
            None => return Err("&".to_string() + sub)
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
    PINode(String)
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
    pub attributes: HashMap<(String, Option<String>), String>
}

#[derive(PartialEq, Eq, Debug)]
/// Structure describing a closing tag
pub struct EndTag {
    /// The tag's name
    pub name: String,
    /// The tag's namespace
    pub ns: Option<String>,
    /// The tag's prefix
    pub prefix: Option<String>
}

impl fmt::Display for Xml {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Xml::ElementNode(ref elem) => elem.fmt(f),
            Xml::CharacterNode(ref data) => write!(f, "{}", escape(&data)),
            Xml::CDATANode(ref data) => write!(f, "<![CDATA[{}]]>", &data),
            Xml::CommentNode(ref data) => write!(f, "<!--{}-->", &data),
            Xml::PINode(ref data) => write!(f, "<?{}?>", &data)
        }
    }
}

#[cfg(test)]
mod lib_tests {
    use super::{Xml, Element, escape, unescape};

    #[test]
    fn test_escape() {
        let esc = escape("&<>'\"");
        assert_eq!(esc, "&amp;&lt;&gt;&apos;&quot;");
    }

    #[test]
    fn test_unescape() {
        let unesc = unescape("&amp;lt;&lt;&gt;&apos;&quot;&#x201c;&#x201d;&#38;&#34;");
        assert_eq!(unesc.as_ref().map(|x| &x[..]), Ok("&lt;<>'\"\u{201c}\u{201d}&\""));
    }

    #[test]
    fn test_unescape_invalid() {
        let unesc = unescape("&amp;&nbsp;");
        assert_eq!(unesc.as_ref().map_err(|x| &x[..]), Err("&nbsp;"));
    }

    #[test]
    fn test_show_element() {
        let elem = Element::new("a".to_string(), None, vec![]);
        assert_eq!(format!("{}", elem), "<a/>");

        let elem = Element::new("a".to_string(), None,
                                vec![("href".to_string(), None,
                                      "http://rust-lang.org".to_string())]);
        assert_eq!(format!("{}", elem), "<a href='http://rust-lang.org'/>");

        let mut elem = Element::new("a".to_string(), None, vec![]);
        elem.tag(Element::new("b".to_string(), None, vec![]));
        assert_eq!(format!("{}", elem), "<a><b/></a>");

        let mut elem = Element::new("a".to_string(), None,
                                    vec![("href".to_string(), None,
                                          "http://rust-lang.org".to_string())]);
        elem.tag(Element::new("b".to_string(), None, vec![]));
        assert_eq!(format!("{}", elem), "<a href='http://rust-lang.org'><b/></a>");
    }

    #[test]
    fn test_show_element_xmlns() {
        let elem: Element = "<a xmlns='urn:test'/>".parse().unwrap();
        assert_eq!(format!("{}", elem), "<a xmlns='urn:test'/>");

        let elem: Element = "<a xmlns='urn:test'><b xmlns='urn:toast'/></a>".parse().unwrap();
        assert_eq!(format!("{}", elem), "<a xmlns='urn:test'><b xmlns='urn:toast'/></a>");

        let elem = Element::new("a".to_string(), Some("urn:test".to_string()),
                                vec![("href".to_string(), None,
                                      "http://rust-lang.org".to_string())]);
        assert_eq!(format!("{}", elem), "<a xmlns='urn:test' href='http://rust-lang.org'/>");
    }

    #[test]
    fn test_show_characters() {
        let chars = Xml::CharacterNode("some text".to_string());
        assert_eq!(format!("{}", chars), "some text");
    }

    #[test]
    fn test_show_cdata() {
        let chars = Xml::CDATANode("some text".to_string());
        assert_eq!(format!("{}", chars), "<![CDATA[some text]]>");
    }

    #[test]
    fn test_show_comment() {
        let chars = Xml::CommentNode("some text".to_string());
        assert_eq!(format!("{}", chars), "<!--some text-->");
    }

    #[test]
    fn test_show_pi() {
        let chars = Xml::PINode("xml version='1.0'".to_string());
        assert_eq!(format!("{}", chars), "<?xml version='1.0'?>");
    }

    #[test]
    fn test_content_str() {
        let mut elem = Element::new("a".to_string(), None, vec![]);
        elem.pi("processing information".to_string())
            .cdata("<hello/>".to_string())
            .tag_stay(Element::new("b".to_string(), None, vec![]))
            .text("World".to_string())
            .comment("Nothing to see".to_string());
        assert_eq!(elem.content_str(), "<hello/>World");
    }
}

#[cfg(test)]
mod lib_bench {
    extern crate test;

    use std::iter::repeat;
    use self::test::Bencher;
    use super::{escape, unescape};

    #[bench]
    fn bench_escape(bh: &mut Bencher) {
        let input: String = repeat("&<>'\"").take(100).collect();
        bh.iter(|| {
            escape(&input)
        });
        bh.bytes = input.len() as u64;
    }

    #[bench]
    fn bench_unescape(bh: &mut Bencher) {
        let input: String = repeat("&amp;&lt;&gt;&apos;&quot;").take(50).collect();
        bh.iter(|| {
            unescape(&input)
        });
        bh.bytes = input.len() as u64;
    }
}
