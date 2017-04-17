RustyXML
========

[![Build Status](https://travis-ci.org/Florob/RustyXML.svg?branch=master)](https://travis-ci.org/Florob/RustyXML)

[Documentation](https://docs.babelmonkeys.de/RustyXML/xml)

RustyXML is a namespace aware XML parser written in Rust.
Right now it provides a basic SAX-like API, and an ElementBuilder based on that.

The parser itself is derived from OFXMLParser as found in ObjFW
<https://webkeks.org/objfw/>.

The current limitations are:
* Incomplete error checking
* Unstable API

This project tracks Rust stable.

Examples
--------
Parse a string into an `Element` struct:
```rust
use xml::Element;

let p: Element = "<p><a href='//example.com'/></p>".parse().unwrap();
let href = p.get_child("a", None).unwrap().get_attribute("href", None).unwrap();
```

Get events from parsing string data:
```rust
use xml::{Event, Parser};

// Create a new Parser
let mut p = Parser::new();

// Feed data to be parsed
p.feed_str("<a href");
p.feed_str("='//example.com'/>");

// Get events for the fed data
for event in p {
    match event.unwrap() {
        Event::ElementStart(tag) => println!("<{}>", tag.name),
        Event::ElementEnd(tag) => println!("</{}>", tag.name),
        _ => ()
    }
}
```

This should print:
```
<a>
</a>
```

Build `Element`s from `Parser` `Event`s:
```rust
use xml::{Parser, ElementBuilder};

let mut p = xml::Parser::new();
let mut e = xml::ElementBuilder::new();

p.feed_str("<a href='//example.com'/>");
for elem in p.filter_map(|x| e.handle_event(x)) {
    match elem {
        Ok(e) => println!("{}", e),
        Err(e) => println!("{}", e),
    }
}
```

Build `Element`s by hand:
```rust
let mut reply = xml::Element::new("iq".into(), Some("jabber:client".into()),
                                  vec![("type".into(), None, "error".into()),
                                       ("id".into(), None, "42".into())]);
reply.tag(xml::Element::new("error".into(), Some("jabber:client".into()),
                            vec![("type".into(), None, "cancel".into())]))
     .tag_stay(xml::Element::new("forbidden".into(),
                                 Some("urn:ietf:params:xml:ns:xmpp-stanzas".into()),
                                 vec![]))
     .tag(xml::Element::new("text".into(),
                            Some("urn:ietf:params:xml:ns:xmpp-stanzas".into()),
                            vec![]))
     .text("Permission denied".into());
```
Result (some whitespace added for readability):
```xml
<iq xmlns='jabber:client' id='42' type='error'>
  <error type='cancel'>
    <forbidden xmlns='urn:ietf:params:xml:ns:xmpp-stanzas'/>
    <text xmlns='urn:ietf:params:xml:ns:xmpp-stanzas'>Permission denied</text>
  </error>
</iq>
```

License
-------

This project is MIT licensed.
Please see the COPYING file for more information.
