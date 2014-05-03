// RustyXML
// Copyright (c) 2013, 2014 Florian Zeitz
//
// This project is MIT licensed.
// Please see the COPYING file for more information.

use std::fmt;
use std::fmt::Show;
use std::char;
use std::num;
use collections::HashMap;

// General functions

#[inline]
/// Escapes ', ", &, <, and > with the appropriate XML entities.
pub fn escape(input: &str) -> ~str {
    let mut result = StrBuf::with_capacity(input.len());

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
    result.into_owned()
}

#[inline]
/// Unescapes all valid XML entities in a string.
pub fn unescape(input: &str) -> Result<~str, ~str> {
    let mut result = StrBuf::with_capacity(input.len());

    let mut ent = StrBuf::new();
    let mut in_entity = false;
    for c in input.chars() {
        if !in_entity {
            if c != '&' {
                result.push_char(c);
            } else {
                ent = StrBuf::from_str("&");
                in_entity = true;
            }
            continue;
        }

        ent.push_char(c);
        if c == ';' {
            match ent.as_slice() {
                "&quot;" => result.push_char('"'),
                "&apos;" => result.push_char('\''),
                "&gt;"   => result.push_char('>'),
                "&lt;"   => result.push_char('<'),
                "&amp;"  => result.push_char('&'),
                ent => {
                    let len = ent.len();
                    let val = if ent.starts_with("&#x") {
                        num::from_str_radix(ent.slice(3, len-1), 16)
                    } else if ent.starts_with("&#") {
                        num::from_str_radix(ent.slice(2, len-1), 10)
                    } else {
                        None
                    };
                    match val.and_then(|x| char::from_u32(x)) {
                        Some(c) => {
                            result.push_char(c);
                        },
                        None => {
                            println!("{}", ent);
                            return Err(ent.into_owned())
                        }
                    }
                }
            }
            in_entity = false;
        }
    }
    Ok(result.into_owned())
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
    pub name: ~str,
    /// The element's namespace
    pub ns: Option<~str>,
    /// The element's default namespace
    pub default_ns: Option<~str>,
    /// The prefixes set for known namespaces
    pub prefixes: HashMap<~str, ~str>,
    /// The element's `Attribute`s
    pub attributes: Vec<Attribute>,
    /// The element's child `XML` nodes
    pub children: Vec<XML>,
}

#[deriving(Clone,Eq,Show)]
/// A struct representing an XML attribute
pub struct Attribute {
    /// The attribute's name
    pub name: ~str,
    /// The attribute's namespace
    pub ns: Option<~str>,
    /// The attribute's value
    pub value: ~str
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
    pub name: ~str,
    /// The tag's namespace
    pub ns: Option<~str>,
    /// The tag's prefix
    pub prefix: Option<~str>,
    /// Attributes included in the tag
    pub attributes: Vec<Attribute>
}

#[deriving(Eq, Show)]
/// Structure describint n closing tag
pub struct EndTag {
    /// The tag's name
    pub name: ~str,
    /// The tag's namespace
    pub ns: Option<~str>,
    /// The tag's prefix
    pub prefix: Option<~str>
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
    all_prefixes.extend(elem.prefixes.iter().map(|(k, v)| (k.clone(), v.clone()) ));

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
        let mut res = StrBuf::new();
        for child in self.children.iter() {
            match *child {
                Element(ref elem) => res.push_str(elem.content_str()),
                CharacterNode(ref data)
                | CDATANode(ref data) => res.push_str(*data),
                _ => ()
            }
        }
        res.into_owned()
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
