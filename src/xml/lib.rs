#![crate_id = "xml#0.1"]
#![crate_type = "lib" ]
#![forbid(non_camel_case_types)]
#![warn(missing_doc)]

/*!
  An XML parsing library
  */

extern crate collections;

pub use base::{escape, unescape};
pub use base::{XML, Element, Attribute, ElementNode, CharacterNode, CDATANode, CommentNode, PINode};
pub use base::{Event, PI, StartTag, EndTag, Characters, CDATA, Comment};
pub use Parser::Error;
pub use Parser::Parser;
pub use ElementBuilder::ElementBuilder;

use std::from_str::FromStr;
mod base;
mod Parser;
mod ElementBuilder;

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
