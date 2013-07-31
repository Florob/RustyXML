use std::str;

// General functions
pub fn escape(input: &str) -> ~str {
    let tmp = str::replace(input, "&", "&amp;");
    let tmp = str::replace(tmp, "<", "&lt;");
    let tmp = str::replace(tmp, ">", "&gt;");
    let tmp = str::replace(tmp, "'", "&apos;");
    str::replace(tmp, "\"", "&quot;")
}

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
}
