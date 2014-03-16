// RustyXML
// Copyright (c) 2013, 2014 Florian Zeitz
//
// This project is MIT licensed.
// Please see the COPYING file for more information.

use std::str;
use std::fmt;
use std::fmt::Show;
use std::vec_ng::Vec;
use collections::HashMap;

// General functions

#[inline]
/// Escapes ', ", &, <, and > with the appropriate XML entities.
pub fn escape(input: &str) -> ~str {
    let mut result = str::with_capacity(input.len());

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
pub fn unescape(input: &str) -> Result<~str, ~str> {
    let mut result = str::with_capacity(input.len());

    let mut ent = ~"";
    let mut in_entity = false;
    for c in input.chars() {
        if !in_entity {
            if c != '&' {
                result.push_char(c);
            } else {
                ent = ~"&";
                in_entity = true;
            }
            continue;
        }

        ent.push_char(c);
        if c == ';' {
            if "&quot;" == ent {
                result.push_char('"');
            } else if "&apos;" == ent {
                result.push_char('\'');
            } else if "&gt;" == ent {
                result.push_char('>');
            } else if "&lt;" == ent {
                result.push_char('<');
            } else if "&amp;" == ent {
                result.push_char('&');
            } else {
                return Err(ent);
            }
            in_entity = false;
        }
    }
    Ok(result)
}

// General types
#[deriving(Clone,Eq)]
/// An Enum describing a XML Node
pub enum XML {
    /// An XML Element
    Element(Element),
    /// Character Data
    CharacterNode(~str),
    /// CDATA
    CDATANode(~str),
    /// A XML Comment
    CommentNode(~str),
    /// Processing Information
    PINode(~str)
}

#[deriving(Clone,Eq)]
/// A struct representing an XML element
pub struct Element {
    /// The element's name
    name: ~str,
    /// The element's namespace
    ns: Option<~str>,
    /// The element's default namespace
    default_ns: Option<~str>,
    /// The prefixes set for known namespaces
    prefixes: HashMap<~str, ~str>,
    /// The element's `Attribute`s
    attributes: Vec<Attribute>,
    /// The element's child `XML` nodes
    children: Vec<XML>,
}

#[deriving(Clone,Eq,Show)]
/// A struct representing an XML attribute
pub struct Attribute {
    /// The attribute's name
    name: ~str,
    /// The attribute's namespace
    ns: Option<~str>,
    /// The attribute's value
    value: ~str
}

#[deriving(Eq, Show)]
/// Events returned by the `Parser`
pub enum Event {
    /// Event indicating processing information was found
    PI(~str),
    /// Event indicating a start tag was found
    StartTag(StartTag),
    /// Event indicating a end tag was found
    EndTag(EndTag),
    /// Event indicating character data was found
    Characters(~str),
    /// Event indicating CDATA was found
    CDATA(~str),
    /// Event indicating a comment was found
    Comment(~str)
}

#[deriving(Eq, Show)]
/// Structure describint an opening tag
pub struct StartTag {
    /// The tag's name
    name: ~str,
    /// The tag's namespace
    ns: Option<~str>,
    /// The tag's prefix
    prefix: Option<~str>,
    /// Attributes included in the tag
    attributes: Vec<Attribute>
}

#[deriving(Eq, Show)]
/// Structure describint n closing tag
pub struct EndTag {
    /// The tag's name
    name: ~str,
    /// The tag's namespace
    ns: Option<~str>,
    /// The tag's prefix
    prefix: Option<~str>
}

#[deriving(Eq, Show)]
/// If an error occurs while parsing some XML, this is the structure which is
/// returned
pub struct Error {
    /// The line number at which the error occurred
    line: uint,
    /// The column number at which the error occurred
    col: uint,
    /// A message describing the type of the error
    msg: ~str
}

impl Show for XML {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Element(ref elem) => elem.fmt(f),
            CharacterNode(ref data) => write!(f.buf, "{}", escape(*data)),
            CDATANode(ref data) => write!(f.buf, "<![CDATA[{}]]>", *data),
            CommentNode(ref data) => write!(f.buf, "<!--{}-->", *data),
            PINode(ref data) => write!(f.buf, "<?{}?>", *data)
        }
    }
}

fn fmt_elem(elem: &Element, parent: Option<&Element>, all_prefixes: &HashMap<~str, ~str>,
            f: &mut fmt::Formatter) -> fmt::Result {
    let mut all_prefixes = all_prefixes.clone();
    all_prefixes.extend(&mut elem.prefixes.iter().map(|(k, v)| (k.clone(), v.clone()) ));

    // Do we need a prefix?
    try!(if elem.ns != elem.default_ns {
        let prefix = all_prefixes.find(elem.ns.get_ref()).expect("No namespace prefix bound");
        write!(f.buf, "<{}:{}", *prefix, elem.name)
    } else {
        write!(f.buf, "<{}", elem.name)
    });

    // Do we need to set the default namespace ?
    if (parent.is_none() && elem.default_ns.is_some()) ||
       (parent.is_some() && parent.unwrap().default_ns != elem.default_ns) {
        try!(match elem.default_ns {
            None => write!(f.buf, " xmlns=''"),
            Some(ref x) => write!(f.buf, " xmlns='{}'", *x)
        });
    }

    for attr in elem.attributes.iter() {
        try!(match attr.ns {
            Some(ref ns) => {
                let prefix = all_prefixes.find(ns).expect("No namespace prefix bound");
                write!(f.buf, " {}:{}='{}'", *prefix, attr.name, escape(attr.value))
            }
            None => write!(f.buf, " {}='{}'", attr.name, escape(attr.value))
        });
    }

    if elem.children.len() == 0 {
        write!(f.buf, "/>")
    } else {
        try!(write!(f.buf, ">"));
        for child in elem.children.iter() {
            try!(match *child {
                Element(ref child) => fmt_elem(child, Some(elem), &all_prefixes, f),
                ref o => o.fmt(f)
            });
        }
        if elem.ns != elem.default_ns {
            let prefix = all_prefixes.find(elem.ns.get_ref()).expect("No namespace prefix bound");
            write!(f.buf, "</{}:{}>", *prefix, elem.name)
        } else {
            write!(f.buf, "</{}>", elem.name)
        }
    }
}

impl Show for Element{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt_elem(self, None, &HashMap::new(), f)
    }
}

impl Element {
    /// Returns the character and CDATA contained in the element.
    pub fn content_str(&self) -> ~str {
        let mut res = ~"";
        for child in self.children.iter() {
            match *child {
                Element(ref elem) => res.push_str(elem.content_str()),
                CharacterNode(ref data)
                | CDATANode(ref data) => res.push_str(*data),
                _ => ()
            }
        }
        res
    }

    /// Gets an `Attribute` with the specified name. When an attribute with the
    /// specified name does not exist `None` is returned.
    pub fn attribute_with_name<'a>(&'a self, name: &str) -> Option<&'a Attribute> {
        self.attribute_with_name_and_ns(name, None)
    }

    /// Gets an `Attribute` with the specified name and namespace. When an attribute with the
    /// specified name does not exist `None` is returned.
    pub fn attribute_with_name_and_ns<'a>(&'a self, name: &str, ns: Option<~str>)
      -> Option<&'a Attribute> {
        for attr in self.attributes.iter() {
            if name == attr.name && ns == attr.ns {
                return Some(attr);
            }
        }
        None
    }

    /// Gets the first child `Element` with the specified name. When no child
    /// with the specified name exists `None` is returned.
    pub fn child_with_name<'a>(&'a self, name: &str) -> Option<&'a Element> {
        self.child_with_name_and_ns(name, None)
    }

    /// Gets the first child `Element` with the specified name and namespace. When no child
    /// with the specified name exists `None` is returned.
    pub fn child_with_name_and_ns<'a>(&'a self, name: &str, ns: Option<~str>)
      -> Option<&'a Element> {
        for child in self.children.iter() {
            match *child {
                Element(ref elem) if name == elem.name && ns == elem.ns => return Some(&*elem),
                _ => ()
            }
        }
        None
    }

    /// Get all children `Element` with the specified name. When no child
    /// with the specified name exists an empty vetor is returned.
    pub fn children_with_name<'a>(&'a self, name: &str) -> Vec<&'a Element> {
        self.children_with_name_and_ns(name, None)
    }

    /// Get all children `Element` with the specified name and namespace. When no child
    /// with the specified name exists an empty vetor is returned.
    pub fn children_with_name_and_ns<'a>(&'a self, name: &str, ns: Option<~str>)
      -> Vec<&'a Element> {
        let mut res: Vec<&'a Element> = Vec::new();
        for child in self.children.iter() {
            match *child {
                Element(ref elem) if name == elem.name && ns == elem.ns => res.push(&*elem),
                _ => ()
            }
        }
        res
    }
}
