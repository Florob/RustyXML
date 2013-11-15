// RustyXML
// Copyright (c) 2013 Florian Zeitz
//
// This project is MIT licensed.
// Please see the COPYING file for more information.

use std::str;
use std::to_str::ToStr;

condition! {
    pub unrecognized_entity: (~str) -> ~str;
}

// General functions

#[inline]
/// Escapes ', ", &, <, and > with the appropriate XML entities.
pub fn escape(input: &str) -> ~str {
    let mut result = str::with_capacity(input.len());

    for c in input.iter() {
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
pub fn unescape(input: &str) -> ~str {
    let mut result = str::with_capacity(input.len());

    let mut ent = ~"";
    let mut in_entity = false;
    for c in input.iter() {
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
            let ent_s = ent.as_slice();
            if ent_s == "&quot;" {
                result.push_char('"');
            } else if ent_s == "&apos;" {
                result.push_char('\'');
            } else if ent_s == "&gt;" {
                result.push_char('>');
            } else if ent_s == "&lt;" {
                result.push_char('<');
            } else if ent_s == "&amp;" {
                result.push_char('&');
            } else {
                result.push_str(unrecognized_entity::cond.raise(ent.clone()));
            }
            in_entity = false;
        }
    }
    result
}

// General types
#[deriving(Clone,Eq)]
/// An Enum describing a XML Node
pub enum XML {
    /// An XML Element
    Element(~Element),
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
    /// The element's `Attribute`s
    attributes: ~[Attribute],
    /// The element's child `XML` nodes
    children: ~[XML]
}

#[deriving(Clone,Eq)]
/// A struct representing an XML attribute
pub struct Attribute {
    /// The attribute's name
    name: ~str,
    /// The attribute's value
    value: ~str
}

#[deriving(Eq)]
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

#[deriving(Eq)]
/// Structure describint an opening tag
pub struct StartTag {
    /// The tag's name
    name: ~str,
    /// Attributes included in the tag
    attributes: ~[Attribute]
}

#[deriving(Eq)]
/// Structure describint n closing tag
pub struct EndTag {
    /// The tag's name
    name: ~str
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
    msg: ~str
}

impl ToStr for XML {
    /// Returns a string representation of the XML Node.
    fn to_str(&self) -> ~str {
        match *self {
            Element(ref elem) => elem.to_str(),
            CharacterNode(ref data) => escape(*data),
            CDATANode(ref data) => format!("<![CDATA[{}]]>", *data),
            CommentNode(ref data) => format!("<!--{}-->", *data),
            PINode(ref data) => format!("<?{}?>", *data)
        }
    }
}

impl ToStr for Element {
    /// Returns a string representation of the XML Element.
    fn to_str(&self) -> ~str {
        let mut res = "<" + self.name;

        for attr in self.attributes.iter() {
            res.push_str(format!(" {}='{}'", attr.name, escape(attr.value)));
        }

        if self.children.len() == 0 {
            res.push_str("/>");
        } else {
            res.push_str(">");
            for child in self.children.iter() {
                res.push_str(child.to_str());
            }
            res.push_str(format!("</{}>", self.name));
        }
        res
    }
}

impl Element {
    /// Returns the character and CDATA conatined in the element.
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
        for i in range(0, self.attributes.len()) {
            let attr: &'a Attribute = &self.attributes[i];
            if name == attr.name {
                return Some(attr);
            }
        }
        None
    }

    /// Gets the first child `Element` with the specified name. When no child
    /// with the specified name exists `None` is returned.
    pub fn child_with_name<'a>(&'a self, name: &str) -> Option<&'a Element> {
        for i in range(0, self.children.len()) {
            let child: &'a XML = &self.children[i];
            match *child {
                Element(ref elem) if name == elem.name => return Some(&**elem),
                _ => ()
            }
        }
        None
    }

    /// Get all children `Element` with the specified name. When no child
    /// with the specified name exists an empty vetor is returned.
    pub fn children_with_name<'a>(&'a self, name: &str) -> ~[&'a Element] {
        let mut res: ~[&'a Element] = ~[];
        for i in range(0, self.children.len()) {
            let child: &'a XML = &self.children[i];
            match *child {
                Element(ref elem) if name == elem.name => res.push(&**elem),
                _ => ()
            }
        }
        res
    }
}
