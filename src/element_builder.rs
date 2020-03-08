// RustyXML
// Copyright 2013-2016 RustyXML developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use super::{Element, EndTag, Event, StartTag, Xml};
use crate::parser::ParserError;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(PartialEq, Debug, Clone)]
/// The structure returned for errors encountered while building an `Element`
pub enum BuilderError {
    /// Errors encountered by the `Parser`
    Parser(ParserError),
    /// Elements were improperly nested, e.g. <a><b></a></b>
    ImproperNesting,
    /// No element was found
    NoElement,
}

impl Error for BuilderError {
    fn description(&self) -> &str {
        match *self {
            BuilderError::Parser(ref err) => err.description(),
            BuilderError::ImproperNesting => "Elements not properly nested",
            BuilderError::NoElement => "No elements found",
        }
    }

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            BuilderError::Parser(ref err) => Some(err),
            _ => None,
        }
    }
}

impl fmt::Display for BuilderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BuilderError::Parser(ref err) => err.fmt(f),
            BuilderError::ImproperNesting => write!(f, "Elements not properly nested"),
            BuilderError::NoElement => write!(f, "No elements found"),
        }
    }
}

impl From<ParserError> for BuilderError {
    fn from(err: ParserError) -> BuilderError {
        BuilderError::Parser(err)
    }
}

/// An Element Builder, building `Element`s from `Event`s as produced by `Parser`
///
/// ~~~
/// use xml::{Parser, ElementBuilder};
///
/// let mut parser = Parser::new();
/// let mut builder = ElementBuilder::new();
///
/// parser.feed_str("<example/>");
/// for result in parser.filter_map(|event| builder.handle_event(event)) {
///     println!("{}", result.unwrap());
/// }
/// ~~~
pub struct ElementBuilder {
    stack: Vec<Element>,
    default_ns: Vec<Option<String>>,
    prefixes: HashMap<String, String>,
}

impl ElementBuilder {
    /// Returns a new `ElementBuilder`
    pub fn new() -> ElementBuilder {
        let mut prefixes = HashMap::with_capacity(2);
        prefixes.insert(
            "http://www.w3.org/XML/1998/namespace".to_owned(),
            "xml".to_owned(),
        );
        prefixes.insert(
            "http://www.w3.org/2000/xmlns/".to_owned(),
            "xmlns".to_owned(),
        );
        ElementBuilder {
            stack: Vec::new(),
            default_ns: Vec::new(),
            prefixes,
        }
    }

    /// Bind a prefix to a namespace
    pub fn define_prefix(&mut self, prefix: String, ns: String) {
        self.prefixes.insert(ns, prefix);
    }

    /// Set the default namespace
    pub fn set_default_ns(&mut self, ns: String) {
        self.default_ns = vec![Some(ns)];
    }

    /// Let the builder process an `Event` to ultimately build an `Element`.
    ///
    /// While no root element has been finished `None` is returned.
    /// Once sufficent data has been received an `Element` is returned as `Some(Ok(elem))`.
    /// Upon Error `Some(Err("message"))` is returned.
    pub fn handle_event(
        &mut self,
        e: Result<Event, ParserError>,
    ) -> Option<Result<Element, BuilderError>> {
        let e = match e {
            Ok(o) => o,
            Err(e) => return Some(Err(From::from(e))),
        };
        match e {
            Event::PI(cont) => {
                if let Some(elem) = self.stack.last_mut() {
                    elem.children.push(Xml::PINode(cont));
                }
            }
            Event::ElementStart(StartTag {
                name,
                ns,
                prefix: _,
                attributes,
            }) => {
                let mut elem = Element {
                    name,
                    ns,
                    default_ns: None,
                    prefixes: self.prefixes.clone(),
                    attributes,
                    children: Vec::new(),
                };

                if let Some(default) = self.default_ns.last().cloned() {
                    self.default_ns.push(default)
                }

                for (&(ref name, ref ns), value) in &elem.attributes {
                    if ns.is_none() && name == "xmlns" {
                        self.default_ns.pop();
                        if value.is_empty() {
                            self.default_ns.push(None);
                        } else {
                            self.default_ns.push(Some(value.clone()));
                        }
                        continue;
                    }

                    if ns
                        .as_ref()
                        .map_or(false, |x| x == "http://www.w3.org/2000/xmlns/")
                    {
                        elem.prefixes.insert(value.clone(), name.clone());
                    }
                }
                elem.default_ns = self.default_ns.last().unwrap_or(&None).clone();

                self.stack.push(elem);
            }
            Event::ElementEnd(EndTag {
                name,
                ns,
                prefix: _,
            }) => {
                let elem = match self.stack.pop() {
                    Some(elem) => elem,
                    None => return Some(Err(BuilderError::ImproperNesting)),
                };
                self.default_ns.pop();
                if elem.name != name || elem.ns != ns {
                    return Some(Err(BuilderError::ImproperNesting));
                } else {
                    match self.stack.last_mut() {
                        Some(e) => e.children.push(Xml::ElementNode(elem)),
                        None => return Some(Ok(elem)),
                    }
                }
            }
            Event::Characters(chars) => {
                if let Some(elem) = self.stack.last_mut() {
                    elem.children.push(Xml::CharacterNode(chars));
                }
            }
            Event::CDATA(chars) => {
                if let Some(elem) = self.stack.last_mut() {
                    elem.children.push(Xml::CDATANode(chars));
                }
            }
            Event::Comment(cont) => {
                if let Some(elem) = self.stack.last_mut() {
                    elem.children.push(Xml::CommentNode(cont));
                }
            }
        }
        None
    }
}
