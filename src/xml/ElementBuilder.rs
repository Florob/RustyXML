// RustyXML
// Copyright (c) 2013 Florian Zeitz
//
// This project is MIT licensed.
// Please see the COPYING file for more information.

use super::{Event, PI, StartTag, EndTag, Characters, CDATA, Comment};
use super::{Element, CharacterNode, CDATANode, CommentNode, PINode};
use std::hashmap::HashMap;

// DOM Builder
/// An ELement Builder, building `Element`s from `Event`s as produced by `Parser`
pub struct ElementBuilder {
    priv stack: ~[Element],
    priv default_ns: ~[Option<~str>],
    priv prefixes: HashMap<~str, ~str>
}

impl ElementBuilder {
    /// Returns a new `ElementBuilder`
    pub fn new() -> ElementBuilder {
        let mut e = ElementBuilder {
            stack: ~[],
            default_ns: ~[],
            prefixes: HashMap::with_capacity(2),
        };
        e.prefixes.swap(~"http://www.w3.org/XML/1998/namespace", ~"xml");
        e.prefixes.swap(~"http://www.w3.org/2000/xmlns/", ~"xmlns");
        e
    }

    /// Bind a prefix to a namespace
    pub fn define_prefix(&mut self, prefix: ~str, ns: ~str) {
        self.prefixes.swap(ns, prefix);
    }

    pub fn set_default_ns(&mut self, ns: ~str) {
        self.default_ns = ~[Some(ns)];
    }

    /// Hands an `Event` to the builder.
    /// While no root element has been finished `Ok(None)` is returned.
    /// Once sufficent data has been received an `Element` is returned as `Ok(elem)`.
    /// Upon Error `Err("message")` is returned.
    pub fn push_event(&mut self, e: Event) -> Result<Option<Element>, ~str> {
        match e {
            PI(cont) => {
                let l = self.stack.len();
                if l > 0 {
                    self.stack[l-1].children.push(PINode(cont));
                }
                Ok(None)
            }
            StartTag(StartTag { name, ns, prefix: _, attributes }) => {
                let mut elem = Element {
                    name: name.clone(),
                    ns: ns.clone(),
                    default_ns: None,
                    prefixes: self.prefixes.clone(),
                    attributes: ~[],
                    children: ~[]
                };

                if !self.default_ns.is_empty() {
                    let cur_default = self.default_ns.last().clone();
                    self.default_ns.push(cur_default);
                }

                for attr in attributes.iter() {
                    if attr.ns == None && attr.name.as_slice() == "xmlns" {
                        self.default_ns.pop_opt();
                        if attr.value.len() == 0 {
                            self.default_ns.push(None);
                        } else {
                            self.default_ns.push(Some(attr.value.clone()));
                        }
                        continue;
                    }
                    if attr.ns == Some(~"http://www.w3.org/2000/xmlns/") {
                        elem.prefixes.swap(attr.value.clone(), attr.name.clone());
                    }
                    elem.attributes.push(attr.clone());
                }
                elem.default_ns = self.default_ns.last_opt().unwrap_or(&None).clone();

                self.stack.push(elem);

                Ok(None)
            }
            EndTag(EndTag { name, ns, prefix: _ }) => {
                if self.stack.len() == 0 {
                    return Err(~"Elements not properly nested");
                }
                self.default_ns.pop_opt();
                let elem = self.stack.pop();
                let l = self.stack.len();
                if elem.name != name || elem.ns != ns {
                    Err(~"Elements not properly nested")
                } else if l == 0 {
                    Ok(Some(elem))
                } else {
                    self.stack[l-1].children.push(Element(elem));
                    Ok(None)
                }
            }
            Characters(chars) => {
                let l = self.stack.len();
                if l > 0 {
                    self.stack[l-1].children.push(CharacterNode(chars));
                }
                Ok(None)
            }
            CDATA(chars) => {
                let l = self.stack.len();
                if l > 0 {
                    self.stack[l-1].children.push(CDATANode(chars));
                }
                Ok(None)
            }
            Comment(cont) => {
                let l = self.stack.len();
                if l > 0 {
                    self.stack[l-1].children.push(CommentNode(cont));
                }
                Ok(None)
            }
        }
    }
}
