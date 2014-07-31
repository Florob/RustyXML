// RustyXML
// Copyright (c) 2013, 2014 Florian Zeitz
//
// This project is MIT licensed.
// Please see the COPYING file for more information.

#![crate_name = "xml"]
#![crate_type = "lib" ]
#![forbid(non_camel_case_types)]
#![warn(missing_doc)]

/*!
  An XML parsing library
  */

extern crate collections;

pub use Parser::Error;
pub use Parser::Parser;
pub use ElementBuilder::ElementBuilder;

use std::fmt;
use std::fmt::Show;
use std::char;
use std::num;
use std::collections::HashMap;
use std::from_str::FromStr;

mod Parser;
mod ElementBuilder;

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
            o => result.push_char(o)
        }
    }
    result
}

#[inline]
/// Unescapes all valid XML entities in a string.
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
                let ent = sub.slice_to(idx);
                match ent {
                    "quot" => result.push_char('"'),
                    "apos" => result.push_char('\''),
                    "gt"   => result.push_char('>'),
                    "lt"   => result.push_char('<'),
                    "amp"  => result.push_char('&'),
                    ent => {
                        let val = if ent.starts_with("#x") {
                            num::from_str_radix(ent.slice_from(2), 16)
                        } else if ent.starts_with("#") {
                            num::from_str_radix(ent.slice_from(1), 10)
                        } else {
                            None
                        };
                        match val.and_then(|x| char::from_u32(x)) {
                            Some(c) => {
                                result.push_char(c);
                            },
                            None => {
                                return Err("&".to_string().append(ent).append(";"))
                            }
                        }
                    }
                }
                result.push_str(sub.slice_from(idx+1));
            }
            None => return Err("&".to_string().append(sub))
        }
    }
    Ok(result)
}

// General types
#[deriving(Clone, PartialEq)]
/// An Enum describing a XML Node
pub enum XML {
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

#[deriving(Clone, PartialEq)]
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
    /// The element's `Attribute`s
    pub attributes: Vec<Attribute>,
    /// The element's child `XML` nodes
    pub children: Vec<XML>,
}

#[deriving(Clone, PartialEq, Eq, Show)]
/// A struct representing an XML attribute
pub struct Attribute {
    /// The attribute's name
    pub name: String,
    /// The attribute's namespace
    pub ns: Option<String>,
    /// The attribute's value
    pub value: String
}

#[deriving(PartialEq, Eq, Show)]
/// Events returned by the `Parser`
pub enum Event {
    /// Event indicating processing information was found
    PI(String),
    /// Event indicating a start tag was found
    StartTag(StartTag),
    /// Event indicating a end tag was found
    EndTag(EndTag),
    /// Event indicating character data was found
    Characters(String),
    /// Event indicating CDATA was found
    CDATA(String),
    /// Event indicating a comment was found
    Comment(String)
}

#[deriving(PartialEq, Eq, Show)]
/// Structure describing an opening tag
pub struct StartTag {
    /// The tag's name
    pub name: String,
    /// The tag's namespace
    pub ns: Option<String>,
    /// The tag's prefix
    pub prefix: Option<String>,
    /// Attributes included in the tag
    pub attributes: Vec<Attribute>
}

#[deriving(PartialEq, Eq, Show)]
/// Structure describing a closing tag
pub struct EndTag {
    /// The tag's name
    pub name: String,
    /// The tag's namespace
    pub ns: Option<String>,
    /// The tag's prefix
    pub prefix: Option<String>
}

impl Show for XML {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ElementNode(ref elem) => elem.fmt(f),
            CharacterNode(ref data) => write!(f, "{}", escape(data.as_slice())),
            CDATANode(ref data) => write!(f, "<![CDATA[{}]]>", data.as_slice()),
            CommentNode(ref data) => write!(f, "<!--{}-->", data.as_slice()),
            PINode(ref data) => write!(f, "<?{}?>", data.as_slice())
        }
    }
}

fn fmt_elem(elem: &Element, parent: Option<&Element>, all_prefixes: &HashMap<String, String>,
            f: &mut fmt::Formatter) -> fmt::Result {
    let mut all_prefixes = all_prefixes.clone();
    all_prefixes.extend(elem.prefixes.iter().map(|(k, v)| (k.clone(), v.clone()) ));

    // Do we need a prefix?
    try!(if elem.ns != elem.default_ns {
        let prefix = all_prefixes.find(elem.ns.get_ref()).expect("No namespace prefix bound");
        write!(f, "<{}:{}", *prefix, elem.name)
    } else {
        write!(f, "<{}", elem.name)
    });

    // Do we need to set the default namespace ?
    match (parent, &elem.default_ns) {
        // No parent, namespace is not empty
        (None, &Some(ref ns)) =>  try!(write!(f, " xmlns='{}'", *ns)),
        // Parent and child namespace differ
        (Some(parent), ns) if !parent.default_ns.eq(ns) => try!(match *ns {
            None => write!(f, " xmlns=''"),
            Some(ref ns) => write!(f, " xmlns='{}'", *ns)
        }),
        _ => ()
    }

    for attr in elem.attributes.iter() {
        try!(match attr.ns {
            Some(ref ns) => {
                let prefix = all_prefixes.find(ns).expect("No namespace prefix bound");
                write!(f, " {}:{}='{}'", *prefix, attr.name, escape(attr.value.as_slice()))
            }
            None => write!(f, " {}='{}'", attr.name, escape(attr.value.as_slice()))
        });
    }

    if elem.children.len() == 0 {
        write!(f, "/>")
    } else {
        try!(write!(f, ">"));
        for child in elem.children.iter() {
            try!(match *child {
                ElementNode(ref child) => fmt_elem(child, Some(elem), &all_prefixes, f),
                ref o => o.fmt(f)
            });
        }
        if elem.ns != elem.default_ns {
            let prefix = all_prefixes.find(elem.ns.get_ref()).expect("No namespace prefix bound");
            write!(f, "</{}:{}>", *prefix, elem.name)
        } else {
            write!(f, "</{}>", elem.name)
        }
    }
}

impl Show for Element{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt_elem(self, None, &HashMap::new(), f)
    }
}

impl Element {
    /// Create a new element
    pub fn new(name: &str, ns: Option<&str>, attrs: Vec<Attribute>) -> Element {
        let ns = ns.map(|x| x.to_string());
        Element {
            name: name.to_string(),
            ns: ns.clone(),
            default_ns: ns,
            prefixes: HashMap::new(),
            attributes: attrs,
            children: Vec::new()
        }
    }

    /// Returns the character and CDATA contained in the element.
    pub fn content_str(&self) -> String {
        let mut res = String::new();
        for child in self.children.iter() {
            match *child {
                ElementNode(ref elem) => res.push_str(elem.content_str().as_slice()),
                CharacterNode(ref data)
                | CDATANode(ref data) => res.push_str(data.as_slice()),
                _ => ()
            }
        }
        res
    }

    /// Gets an `Attribute` with the specified name and namespace. When an attribute with the
    /// specified name does not exist `None` is returned.
    pub fn get_attribute<'a>(&'a self, name: &str, ns: Option<&str>) -> Option<&'a Attribute> {
        let mut it = self.attributes.iter();
        for attr in it {
            if !name.equiv(&attr.name) {
                continue;
            }
            match (ns, attr.ns.as_ref().map(|x| x.as_slice())) {
                (Some(ref x), Some(ref y)) if x == y => return Some(attr),
                (None, None) => return Some(attr),
                _ => continue
            }
        }
        None
    }

    /// Gets the first child `Element` with the specified name and namespace. When no child
    /// with the specified name exists `None` is returned.
    pub fn get_child<'a>(&'a self, name: &str, ns: Option<&str>) -> Option<&'a Element> {
        for child in self.children.iter() {
            match *child {
                ElementNode(ref elem) => {
                    if !name.equiv(&elem.name) {
                        continue;
                    }
                    match (ns, elem.ns.as_ref().map(|x| x.as_slice())) {
                        (Some(ref x), Some(ref y)) if x == y => return Some(&*elem),
                        (None, None) => return Some(&*elem),
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
            match *child {
                ElementNode(ref elem) => {
                    if !name.equiv(&elem.name) {
                        continue;
                    }
                    match (ns, elem.ns.as_ref().map(|x| x.as_slice())) {
                        (Some(ref x), Some(ref y)) if x == y => res.push(&*elem),
                        (None, None) => res.push(&*elem),
                        _ => continue
                    }
                }
                _ => ()
            }
        }
        res
    }
}

impl<'a> Element {
    /// Appends a child element. Returns a reference to the added element.
    pub fn tag(&'a mut self, child: Element) -> &'a mut Element {
        self.children.push(ElementNode(child));
        let error = "Internal error: Could not get reference to new element!";
        let elem = match self.children.mut_last().expect(error) {
            &ElementNode(ref mut elem) => elem,
            _ => fail!(error)
        };
        elem
    }

    /// Appends a child element. Returns a mutable reference to self.
    pub fn tag_stay(&'a mut self, child: Element) -> &'a mut Element {
        self.children.push(ElementNode(child));
        self
    }

    /// Appends characters. Returns a mutable reference to self.
    pub fn text(&'a mut self, text: &str) -> &'a mut Element {
        self.children.push(CharacterNode(text.to_string()));
        self
    }

    /// Appends CDATA. Returns a mutable reference to self.
    pub fn cdata(&'a mut self, text: &str) -> &'a mut Element {
        self.children.push(CDATANode(text.to_string()));
        self
    }

    /// Appends a comment. Returns a mutable reference to self.
    pub fn comment(&'a mut self, text: &str) -> &'a mut Element {
        self.children.push(CommentNode(text.to_string()));
        self
    }

    /// Appends processing information. Returns a mutable reference to self.
    pub fn pi(&'a mut self, text: &str) -> &'a mut Element {
        self.children.push(PINode(text.to_string()));
        self
    }
}

impl FromStr for Element {
    #[inline]
    fn from_str(data: &str) -> Option<Element> {
        let mut p = Parser::Parser::new();
        let mut e = ElementBuilder::ElementBuilder::new();
        let mut result = None;

        p.feed_str(data);
        for event in p {
            match event {
                Ok(event) => match e.push_event(event) {
                    Ok(Some(elem)) => result = Some(elem),
                    _ => ()
                },
                _ => ()
            }
        }
        result
    }
}
