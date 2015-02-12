// RustyXML
// Copyright (c) 2013, 2014 Florian Zeitz
//
// This project is MIT licensed.
// Please see the COPYING file for more information.

#![crate_name = "xml"]
#![crate_type = "lib" ]
#![forbid(non_camel_case_types)]
#![warn(missing_docs)]

// These are unstable for now
#![feature(core)]
#![cfg_attr(test, feature(test))]

/*!
  An XML parsing library
  */

pub use parser::Parser;
pub use parser::ParserError;
pub use element_builder::ElementBuilder;
pub use element_builder::BuilderError;

use std::fmt;
use std::char;
use std::num;
use std::collections::HashMap;
use std::str::FromStr;

mod parser;
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
    for &sub in it.next().iter() {
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
                            num::from_str_radix(&ent[2..], 16).ok()
                        } else if ent.starts_with("#") {
                            num::from_str_radix(&ent[1..], 10).ok()
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
#[derive(Clone, PartialEq)]
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

#[derive(Clone, PartialEq)]
/// A struct representing an XML element
pub struct Element {
    /// The element's name
    pub name: String,
    /// The element's namespace
    pub ns: Option<String>,
    /// The element's default namespace
    pub default_ns: Option<String>,
    /// The prefixes set for known namespaces
    pub prefixes: HashMap<String, String>,
    /// The element's attributes
    pub attributes: HashMap<(String, Option<String>), String>,
    /// The element's child `Xml` nodes
    pub children: Vec<Xml>,
}

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
    Comment(String)
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
            Xml::CharacterNode(ref data) => write!(f, "{}", escape(&data[])),
            Xml::CDATANode(ref data) => write!(f, "<![CDATA[{}]]>", &data[]),
            Xml::CommentNode(ref data) => write!(f, "<!--{}-->", &data[]),
            Xml::PINode(ref data) => write!(f, "<?{}?>", &data[])
        }
    }
}

fn fmt_elem(elem: &Element, parent: Option<&Element>, all_prefixes: &HashMap<String, String>,
            f: &mut fmt::Formatter) -> fmt::Result {
    let mut all_prefixes = all_prefixes.clone();
    all_prefixes.extend(elem.prefixes.iter().map(|(k, v)| (k.clone(), v.clone()) ));

    // Do we need a prefix?
    try!(if elem.ns != elem.default_ns {
        let prefix = all_prefixes.get(elem.ns.as_ref().unwrap_or(&String::new()))
                                 .expect("No namespace prefix bound");
        write!(f, "<{}:{}", *prefix, elem.name)
    } else {
        write!(f, "<{}", elem.name)
    });

    // Do we need to set the default namespace ?
    match (parent, &elem.default_ns) {
        // No parent, namespace is not empty
        (None, &Some(ref ns)) => try!(write!(f, " xmlns='{}'", *ns)),
        // Parent and child namespace differ
        (Some(parent), ns) if !parent.default_ns.eq(ns) => try!(match *ns {
            None => write!(f, " xmlns=''"),
            Some(ref ns) => write!(f, " xmlns='{}'", *ns)
        }),
        _ => ()
    }

    for (&(ref name, ref ns), value) in elem.attributes.iter() {
        try!(match *ns {
            Some(ref ns) => {
                let prefix = all_prefixes.get(ns).expect("No namespace prefix bound");
                write!(f, " {}:{}='{}'", *prefix, name, escape(&value[]))
            }
            None => write!(f, " {}='{}'", name, escape(&value[]))
        });
    }

    if elem.children.len() == 0 {
        write!(f, "/>")
    } else {
        try!(write!(f, ">"));
        for child in elem.children.iter() {
            try!(match *child {
                Xml::ElementNode(ref child) => fmt_elem(child, Some(elem), &all_prefixes, f),
                ref o => fmt::Display::fmt(o, f)
            });
        }
        if elem.ns != elem.default_ns {
            let prefix = all_prefixes.get(elem.ns.as_ref().unwrap())
                                     .expect("No namespace prefix bound");
            write!(f, "</{}:{}>", *prefix, elem.name)
        } else {
            write!(f, "</{}>", elem.name)
        }
    }
}

impl fmt::Display for Element{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt_elem(self, None, &HashMap::new(), f)
    }
}

impl Element {
    /// Create a new element, with specified name and namespace.
    /// Attributes are specified as a slice of (name, namespace, value) tuples.
    pub fn new(name: &str, ns: Option<&str>, attrs: &[(&str, Option<&str>, &str)]) -> Element {
        let ns = ns.map(|x| x.to_string());
        let mut attributes: HashMap<(String, Option<String>), String> = HashMap::new();
        for &(name, ref ns, value) in attrs.iter() {
            attributes.insert((name.to_string(), ns.clone().map(|x| x.to_string())),
                              value.to_string());
        }
        Element {
            name: name.to_string(),
            ns: ns.clone(),
            default_ns: ns,
            prefixes: HashMap::new(),
            attributes: attributes,
            children: Vec::new()
        }
    }

    /// Returns the character and CDATA contained in the element.
    pub fn content_str(&self) -> String {
        let mut res = String::new();
        for child in self.children.iter() {
            match *child {
                Xml::ElementNode(ref elem) => res.push_str(&elem.content_str()[]),
                Xml::CharacterNode(ref data)
                | Xml::CDATANode(ref data) => res.push_str(&data[]),
                _ => ()
            }
        }
        res
    }

    /// Gets an attribute with the specified name and namespace. When an attribute with the
    /// specified name does not exist `None` is returned.
    pub fn get_attribute<'a>(&'a self, name: &str, ns: Option<&str>) -> Option<&'a str> {
        self.attributes.get(&(name.to_string(), ns.map(|x| x.to_string()))).map(|x| &x[])
    }

    /// Sets the attribute with the specified name and namespace.
    /// Returns the original value.
    pub fn set_attribute(&mut self, name: &str, ns: Option<&str>, value: &str) -> Option<String> {
        self.attributes.insert((name.to_string(), ns.map(|x| x.to_string())),
                               value.to_string())
    }

    /// Remove the attribute with the specified name and namespace.
    /// Returns the original value.
    pub fn remove_attribute(&mut self, name: &str, ns: Option<&str>) -> Option<String> {
        self.attributes.remove(&(name.to_string(), ns.map(|x| x.to_string())))
    }

    /// Gets the first child `Element` with the specified name and namespace. When no child
    /// with the specified name exists `None` is returned.
    pub fn get_child<'a>(&'a self, name: &str, ns: Option<&str>) -> Option<&'a Element> {
        for child in self.children.iter() {
            match *child {
                Xml::ElementNode(ref elem) => {
                    if elem.name != name {
                        continue;
                    }
                    match (ns, elem.ns.as_ref().map(|x| &x[])) {
                        (Some(x), Some(y)) if x == y => return Some(elem),
                        (None, None) => return Some(elem),
                        _ => continue
                    }
                }
                _ => ()
            }
        }
        None
    }

    /// Get all children `Element` with the specified name and namespace. When no child
    /// with the specified name exists an empty vetor is returned.
    pub fn get_children<'a>(&'a self, name: &str, ns: Option<&str>) -> Vec<&'a Element> {
        let mut res: Vec<&'a Element> = Vec::new();
        for child in self.children.iter() {
            if let Xml::ElementNode(ref elem) = *child {
                if elem.name != name {
                    continue;
                }
                match (ns, elem.ns.as_ref().map(|x| &x[])) {
                    (Some(x), Some(y)) if x == y => res.push(elem),
                    (None, None) => res.push(elem),
                    _ => continue
                }
            }
        }
        res
    }
}

impl<'a> Element {
    /// Appends a child element. Returns a reference to the added element.
    pub fn tag(&'a mut self, child: Element) -> &'a mut Element {
        self.children.push(Xml::ElementNode(child));
        let error = "Internal error: Could not get reference to new element!";
        let elem = match self.children.last_mut().expect(error) {
            &mut Xml::ElementNode(ref mut elem) => elem,
            _ => panic!(error)
        };
        elem
    }

    /// Appends a child element. Returns a mutable reference to self.
    pub fn tag_stay(&'a mut self, child: Element) -> &'a mut Element {
        self.children.push(Xml::ElementNode(child));
        self
    }

    /// Appends characters. Returns a mutable reference to self.
    pub fn text(&'a mut self, text: &str) -> &'a mut Element {
        self.children.push(Xml::CharacterNode(text.to_string()));
        self
    }

    /// Appends CDATA. Returns a mutable reference to self.
    pub fn cdata(&'a mut self, text: &str) -> &'a mut Element {
        self.children.push(Xml::CDATANode(text.to_string()));
        self
    }

    /// Appends a comment. Returns a mutable reference to self.
    pub fn comment(&'a mut self, text: &str) -> &'a mut Element {
        self.children.push(Xml::CommentNode(text.to_string()));
        self
    }

    /// Appends processing information. Returns a mutable reference to self.
    pub fn pi(&'a mut self, text: &str) -> &'a mut Element {
        self.children.push(Xml::PINode(text.to_string()));
        self
    }
}

impl FromStr for Element {
    type Err = BuilderError;
    #[inline]
    fn from_str(data: &str) -> Result<Element, BuilderError> {
        let mut p = parser::Parser::new();
        let mut e = element_builder::ElementBuilder::new();

        p.feed_str(data);
        for event in p {
            match e.push_event(event) {
                Ok(Some(elem)) => return Ok(elem),
                Ok(None) => (),
                Err(err) => return Err(err)
            }
        }
        Err(BuilderError::NoElement)
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
        assert_eq!(unesc.as_ref().map(|x| &x[]), Ok("&lt;<>'\"\u{201c}\u{201d}&\""));
    }

    #[test]
    fn test_unescape_invalid() {
        let unesc = unescape("&amp;&nbsp;");
        assert_eq!(unesc.as_ref().map_err(|x| &x[]), Err("&nbsp;"));
    }

    #[test]
    fn test_show_element() {
        let elem = Element::new("a", None, &[]);
        assert_eq!(&format!("{}", elem)[], "<a/>");

        let elem = Element::new("a", None, &[("href", None, "http://rust-lang.org")]);
        assert_eq!(&format!("{}", elem)[], "<a href='http://rust-lang.org'/>");

        let mut elem = Element::new("a", None, &[]);
        elem.tag(Element::new("b", None, &[]));
        assert_eq!(&format!("{}", elem)[], "<a><b/></a>");

        let mut elem = Element::new("a", None, &[("href", None, "http://rust-lang.org")]);
        elem.tag(Element::new("b", None, &[]));
        assert_eq!(&format!("{}", elem)[], "<a href='http://rust-lang.org'><b/></a>");
    }

    #[test]
    fn test_show_characters() {
        let chars = Xml::CharacterNode("some text".to_string());
        assert_eq!(&format!("{}", chars)[], "some text");
    }

    #[test]
    fn test_show_cdata() {
        let chars = Xml::CDATANode("some text".to_string());
        assert_eq!(&format!("{}", chars)[], "<![CDATA[some text]]>");
    }

    #[test]
    fn test_show_comment() {
        let chars = Xml::CommentNode("some text".to_string());
        assert_eq!(&format!("{}", chars)[], "<!--some text-->");
    }

    #[test]
    fn test_show_pi() {
        let chars = Xml::PINode("xml version='1.0'".to_string());
        assert_eq!(&format!("{}", chars)[], "<?xml version='1.0'?>");
    }

    #[test]
    fn test_content_str() {
        let mut elem = Element::new("a", None, &[]);
        elem.pi("processing information")
            .cdata("<hello/>")
            .tag_stay(Element::new("b", None, &[]))
            .text("World")
            .comment("Nothing to see");
        assert_eq!(&elem.content_str()[], "<hello/>World");
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
        bh.iter( || {
            escape(&input[])
        });
        bh.bytes = input.len() as u64;
    }

    #[bench]
    fn bench_unescape(bh: &mut Bencher) {
        let input: String = repeat("&amp;&lt;&gt;&apos;&quot;").take(50).collect();
        bh.iter(|| {
            unescape(&input[])
        });
        bh.bytes = input.len() as u64;
    }
}
