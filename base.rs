use std::str;
use std::uint;

// General functions
#[inline]
pub fn escape(input: &str) -> ~str {
    let tmp = str::replace(input, "&", "&amp;");
    let tmp = str::replace(tmp, "<", "&lt;");
    let tmp = str::replace(tmp, ">", "&gt;");
    let tmp = str::replace(tmp, "'", "&apos;");
    str::replace(tmp, "\"", "&quot;")
}

#[inline]
pub fn unescape(input: &str) -> ~str {
    let tmp = str::replace(input, "&quot;", "\"");
    let tmp = str::replace(tmp, "&apos;", "'");
    let tmp = str::replace(tmp, "&gt;", ">");
    let tmp = str::replace(tmp, "&lt;", "<");
    str::replace(tmp, "&amp;", "&")
}

// General types
#[deriving(Clone,Eq)]
pub enum XML {
    Element(~Element),
    CharacterNode(~str),
    CDATANode(~str),
    CommentNode(~str),
    PINode(~str)
}

#[deriving(Clone,Eq)]
pub struct Element {
    name: ~str,
    attributes: ~[Attribute],
    children: ~[XML]
}

#[deriving(Clone,Eq)]
pub struct Attribute {
    name: ~str,
    value: ~str
}

#[deriving(Eq)]
pub enum Event {
    PI(~str),
    StartTag { name: ~str, attributes: ~[Attribute] },
    EndTag { name: ~str },
    Characters(~str),
    CDATA(~str),
    Comment(~str)
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
    msg: @~str
}

impl XML {
    pub fn to_str(&self) -> ~str {
        match *self {
            Element(ref elem) => elem.to_str(),
            CharacterNode(ref data) => escape(*data),
            CDATANode(ref data) => fmt!("<![CDATA[%s]]>", *data),
            CommentNode(ref data) => fmt!("<!--%s-->", *data),
            PINode(ref data) => fmt!("<?%s?>", *data)
        }
    }
}

impl Element {
    pub fn to_str(&self) -> ~str {
        let mut res = fmt!("<%s", self.name);

        for self.attributes.iter().advance |attr| {
            res.push_str(fmt!(" %s='%s'", attr.name, escape(attr.value)));
        }

        if self.children.len() == 0 {
            res.push_str("/>");
        } else {
            res.push_str(">");
            for self.children.iter().advance |child| {
                res.push_str(child.to_str());
            }
            res.push_str(fmt!("</%s>", self.name));
        }
        res
    }

    pub fn content_str(&self) -> ~str {
        let mut res = ~"";
        for self.children.iter().advance |child| {
            match *child {
                Element(ref elem) => res.push_str(elem.content_str()),
                CharacterNode(ref data)
                | CDATANode(ref data) => res.push_str(*data),
                _ => ()
            }
        }
        res
    }

    pub fn attribute_with_name<'a>(&'a self, name: &str) -> Option<&'a Attribute> {
        for uint::range(0, self.attributes.len()) |i| {
            let attr: &'a Attribute = &self.attributes[i];
            if name == attr.name {
                return Some(attr);
            }
        }
        None
    }

    pub fn child_with_name<'a>(&'a self, name: &str) -> Option<&'a Element> {
        for uint::range(0, self.children.len()) |i| {
            let child: &'a XML = &self.children[i];
            match *child {
                Element(ref elem) if name == elem.name => return Some(&**elem),
                _ => ()
            }
        }
        None
    }

    pub fn children_with_name<'a>(&'a self, name: &str) -> ~[&'a Element] {
        let mut res: ~[&'a Element] = ~[];
        for uint::range(0, self.children.len()) |i| {
            let child: &'a XML = &self.children[i];
            match *child {
                Element(ref elem) if name == elem.name => res.push(&**elem),
                _ => ()
            }
        }
        res
    }
}

#[cfg(test)]
priv mod tests {
    use super::*;

    #[test]
    fn test_escape() {
        let esc = escape("&<>'\"");
        assert_eq!(esc, ~"&amp;&lt;&gt;&apos;&quot;");
    }

    #[test]
    fn test_unescape() {
        let unesc = unescape("&amp;lt;&lt;&gt;&apos;&quot;");
        assert_eq!(unesc, ~"&lt;<>'\"");
    }

    #[test]
    fn test_to_str_element() {
        let elem = Element { name: ~"a", attributes: ~[], children: ~[] };
        assert_eq!(elem.to_str(), ~"<a/>");

        let elem = Element {
            name: ~"a",
            attributes: ~[
                Attribute { name: ~"href", value: ~"http://rust-lang.org" }
            ],
            children: ~[]
        };
        assert_eq!(elem.to_str(), ~"<a href='http://rust-lang.org'/>");

        let elem = Element {
            name: ~"a",
            attributes: ~[],
            children: ~[
                Element(~Element { name: ~"b", attributes: ~[], children: ~[] })
            ]
        };
        assert_eq!(elem.to_str(), ~"<a><b/></a>");

        let elem = Element {
            name: ~"a",
            attributes: ~[
                Attribute { name: ~"href", value: ~"http://rust-lang.org" }
            ],
            children: ~[
                Element(~Element { name: ~"b", attributes: ~[], children: ~[] })
            ]
        };
        assert_eq!(elem.to_str(), ~"<a href='http://rust-lang.org'><b/></a>");
    }

    #[test]
    fn test_to_str_characters() {
        let chars = CharacterNode(~"some text");
        assert_eq!(chars.to_str(), ~"some text");
    }

    #[test]
    fn test_to_str_CDATA() {
        let chars = CDATANode(~"some text");
        assert_eq!(chars.to_str(), ~"<![CDATA[some text]]>");
    }

    #[test]
    fn test_to_str_comment() {
        let chars = CommentNode(~"some text");
        assert_eq!(chars.to_str(), ~"<!--some text-->");
    }

    #[test]
    fn test_to_str_pi() {
        let chars = PINode(~"xml version='1.0'");
        assert_eq!(chars.to_str(), ~"<?xml version='1.0'?>");
    }

    #[test]
    fn test_content_str() {
        let elem = Element {
            name: ~"a",
            attributes: ~[],
            children: ~[
                PINode(~"processing information"),
                CDATANode(~"<hello/>"),
                Element(~Element{ name: ~"b", attributes: ~[], children: ~[] }),
                CharacterNode(~"World"),
                CommentNode(~"Nothing to see")
            ]
        };
        assert_eq!(elem.content_str(), ~"<hello/>World");
    }
}
