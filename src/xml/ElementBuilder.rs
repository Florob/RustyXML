// RustyXML
// Copyright (c) 2013, 2014 Florian Zeitz
//
// This project is MIT licensed.
// Please see the COPYING file for more information.

use super::{Event, PI, StartTag, EndTag, Characters, CDATA, Comment};
use super::{Element, ElementNode, CharacterNode, CDATANode, CommentNode, PINode};
use std::collections::HashMap;

// DOM Builder
/// An ELement Builder, building `Element`s from `Event`s as produced by `Parser`
pub struct ElementBuilder {
    stack: Vec<Element>,
    default_ns: Vec<Option<String>>,
    prefixes: HashMap<String, String>
}

impl ElementBuilder {
    /// Returns a new `ElementBuilder`
    pub fn new() -> ElementBuilder {
        let mut prefixes = HashMap::with_capacity(2);
        prefixes.swap("http://www.w3.org/XML/1998/namespace".to_string(), "xml".to_string());
        prefixes.swap("http://www.w3.org/2000/xmlns/".to_string(), "xmlns".to_string());
        ElementBuilder {
            stack: Vec::new(),
            default_ns: Vec::new(),
            prefixes: prefixes
        }
    }

    /// Bind a prefix to a namespace
    pub fn define_prefix(&mut self, prefix: &str, ns: &str) {
        self.prefixes.swap(ns.to_string(), prefix.to_string());
    }

    /// Set the default namespace
    pub fn set_default_ns(&mut self, ns: &str) {
        self.default_ns = vec!(Some(ns.to_string()));
    }

    /// Hands an `Event` to the builder.
    /// While no root element has been finished `Ok(None)` is returned.
    /// Once sufficent data has been received an `Element` is returned as `Ok(elem)`.
    /// Upon Error `Err("message")` is returned.
    pub fn push_event(&mut self, e: Event) -> Result<Option<Element>, &'static str> {
        match e {
            PI(cont) => {
                match self.stack.mut_last() {
                    None => (),
                    Some(elem) => elem.children.push(PINode(cont))
                }
                Ok(None)
            }
            StartTag(StartTag { name, ns, prefix: _, attributes }) => {
                let mut elem = Element {
                    name: name.clone(),
                    ns: ns.clone(),
                    default_ns: None,
                    prefixes: self.prefixes.clone(),
                    attributes: Vec::new(),
                    children: Vec::new()
                };

                if !self.default_ns.is_empty() {
                    let cur_default = self.default_ns.last().unwrap().clone();
                    self.default_ns.push(cur_default);
                }

                for attr in attributes.iter() {
                    if attr.ns == None && attr.name.as_slice() == "xmlns" {
                        self.default_ns.pop();
                        if attr.value.len() == 0 {
                            self.default_ns.push(None);
                        } else {
                            self.default_ns.push(Some(attr.value.clone()));
                        }
                        continue;
                    }
                    if attr.ns == Some("http://www.w3.org/2000/xmlns/".to_string()) {
                        elem.prefixes.swap(attr.value.clone(), attr.name.clone());
                    }
                    elem.attributes.push(attr.clone());
                }
                elem.default_ns = self.default_ns.last().unwrap_or(&None).clone();

                self.stack.push(elem);

                Ok(None)
            }
            EndTag(EndTag { name, ns, prefix: _ }) => {
                let elem = match self.stack.pop() {
                    Some(elem) => elem,
                    None => return Err("Elements not properly nested")
                };
                self.default_ns.pop();
                if elem.name != name || elem.ns != ns {
                    Err("Elements not properly nested")
                } else {
                    match self.stack.mut_last() {
                        None => Ok(Some(elem)),
                        Some(e) => {
                            e.children.push(ElementNode(elem));
                            Ok(None)
                        }
                    }
                }
            }
            Characters(chars) => {
                match self.stack.mut_last() {
                    None => (),
                    Some(elem) => elem.children.push(CharacterNode(chars))
                }
                Ok(None)
            }
            CDATA(chars) => {
                match self.stack.mut_last() {
                    None => (),
                    Some(elem) => elem.children.push(CDATANode(chars))
                }
                Ok(None)
            }
            Comment(cont) => {
                match self.stack.mut_last() {
                    None => (),
                    Some(elem) => elem.children.push(CommentNode(cont))
                }
                Ok(None)
            }
        }
    }
}
