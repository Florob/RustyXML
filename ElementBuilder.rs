// RustyXML
// Copyright (c) 2013 Florian Zeitz
//
// This project is MIT licensed.
// Please see the COPYING file for more information.

use super::{Event, PI, StartTag, EndTag, Characters, CDATA, Comment};
use super::{Element, CharacterNode, CDATANode, CommentNode, PINode};

// DOM Builder
/// An ELement Builder, building `Element`s from `Event`s as produced by `Parser`
pub struct ElementBuilder {
    priv stack: ~[~Element]
}

impl ElementBuilder {
    /// Returns a new `ElementBuilder`
    pub fn new() -> ElementBuilder {
        let e = ElementBuilder {
            stack: ~[]
        };
        e
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
                    (*self.stack[l-1]).children.push(PINode(cont));
                }
                Ok(None)
            }
            StartTag(StartTag { name, attributes }) => {
                self.stack.push(~Element {
                    name: name.clone(),
                    attributes: attributes.clone(),
                    children: ~[]
                });

                Ok(None)
            }
            EndTag(EndTag { name }) => {
                if self.stack.len() == 0 {
                    return Err(~"Elements not properly nested");
                }
                let elem = self.stack.pop();
                let l = self.stack.len();
                if elem.name != name {
                    Err(~"Elements not properly nested")
                } else if l == 0 {
                    Ok(Some(*elem))
                } else {
                    (*self.stack[l-1]).children.push(Element(elem));
                    Ok(None)
                }
            }
            Characters(chars) => {
                let l = self.stack.len();
                if l > 0 {
                    (*self.stack[l-1]).children.push(CharacterNode(chars));
                }
                Ok(None)
            }
            CDATA(chars) => {
                let l = self.stack.len();
                if l > 0 {
                    (*self.stack[l-1]).children.push(CDATANode(chars));
                }
                Ok(None)
            }
            Comment(cont) => {
                let l = self.stack.len();
                if l > 0 {
                    (*self.stack[l-1]).children.push(CommentNode(cont));
                }
                Ok(None)
            }
        }
    }
}
