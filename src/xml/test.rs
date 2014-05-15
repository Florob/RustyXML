extern crate xml;

#[cfg(test)]
mod base_tests {
    extern crate collections;

    use xml::{escape, unescape};
    use xml::{Element, Attribute, CharacterNode, CDATANode, CommentNode, PINode};
    use self::collections::HashMap;

    #[test]
    fn test_escape() {
        let esc = escape("&<>'\"");
        assert_eq!(esc, "&amp;&lt;&gt;&apos;&quot;".to_strbuf());
    }

    #[test]
    fn test_unescape() {
        let unesc = unescape("&amp;lt;&lt;&gt;&apos;&quot;&#x201c;&#x201d;&#38;&#34;");
        assert_eq!(unesc, Ok("&lt;<>'\"\u201c\u201d&\"".to_strbuf()));
    }

    #[test]
    fn test_unescape_invalid() {
        let unesc = unescape("&amp;&nbsp;");
        assert_eq!(unesc, Err("&nbsp;".to_strbuf()));
    }

    #[test]
    fn test_show_element() {
        let elem = Element {
            name: "a".to_strbuf(),
            ns: None,
            default_ns: None,
            prefixes: HashMap::new(),
            attributes: Vec::new(),
            children: Vec::new()
        };
        assert_eq!(format!("{}", elem).as_slice(), "<a/>");

        let elem = Element {
            name: "a".to_strbuf(),
            ns: None,
            default_ns: None,
            prefixes: HashMap::new(),
            attributes: vec!(
                Attribute {
                    name: "href".to_strbuf(),
                    ns: None,
                    value: "http://rust-lang.org".to_strbuf()
                }
            ),
            children: Vec::new()
        };
        assert_eq!(format!("{}", elem).as_slice(), "<a href='http://rust-lang.org'/>");

        let elem = Element {
            name: "a".to_strbuf(),
            ns: None,
            default_ns: None,
            prefixes: HashMap::new(),
            attributes: Vec::new(),
            children: vec!(
                Element(Element {
                    name: "b".to_strbuf(),
                    ns: None,
                    default_ns: None,
                    prefixes: HashMap::new(),
                    attributes: Vec::new(),
                    children: Vec::new()
                })
            )
        };
        assert_eq!(format!("{}", elem).as_slice(), "<a><b/></a>");

        let elem = Element {
            name: "a".to_strbuf(),
            ns: None,
            default_ns: None,
            prefixes: HashMap::new(),
            attributes: vec!(
                Attribute {
                    name: "href".to_strbuf(),
                    ns: None,
                    value: "http://rust-lang.org".to_strbuf()
                }
            ),
            children: vec!(
                Element(Element {
                    name: "b".to_strbuf(),
                    ns: None,
                    default_ns: None,
                    prefixes: HashMap::new(),
                    attributes: Vec::new(),
                    children: Vec::new()
                })
            )
        };
        assert_eq!(format!("{}", elem).as_slice(), "<a href='http://rust-lang.org'><b/></a>");
    }

    #[test]
    fn test_show_characters() {
        let chars = CharacterNode("some text".to_strbuf());
        assert_eq!(format!("{}", chars).as_slice(), "some text");
    }

    #[test]
    fn test_show_CDATA() {
        let chars = CDATANode("some text".to_strbuf());
        assert_eq!(format!("{}", chars).as_slice(), "<![CDATA[some text]]>");
    }

    #[test]
    fn test_show_comment() {
        let chars = CommentNode("some text".to_strbuf());
        assert_eq!(format!("{}", chars).as_slice(), "<!--some text-->");
    }

    #[test]
    fn test_show_pi() {
        let chars = PINode("xml version='1.0'".to_strbuf());
        assert_eq!(format!("{}", chars).as_slice(), "<?xml version='1.0'?>");
    }

    #[test]
    fn test_content_str() {
        let elem = Element {
            name: "a".to_strbuf(),
            ns: None,
            default_ns: None,
            prefixes: HashMap::new(),
            attributes: Vec::new(),
            children: vec!(
                PINode("processing information".to_strbuf()),
                CDATANode("<hello/>".to_strbuf()),
                Element(Element{
                    name: "b".to_strbuf(),
                    ns: None,
                    default_ns: None,
                    prefixes: HashMap::new(),
                    attributes: Vec::new(),
                    children: Vec::new()
                }),
                CharacterNode("World".to_strbuf()),
                CommentNode("Nothing to see".to_strbuf())
            )
        };
        assert_eq!(elem.content_str(), "<hello/>World".to_strbuf());
    }
}

#[cfg(test)]
mod base_bench {
    extern crate test;
    use self::test::Bencher;
    use xml::{escape, unescape};

    #[bench]
    fn bench_escape(bh: &mut Bencher) {
        let input = "&<>'\"".repeat(100);
        bh.iter( || {
            escape(input)
        });
        bh.bytes = input.len() as u64;
    }

    #[bench]
    fn bench_unescape(bh: &mut Bencher) {
        let input = "&amp;&lt;&gt;&apos;&quot;".repeat(50);
        bh.iter(|| {
            unescape(input)
        });
        bh.bytes = input.len() as u64;
    }
}

#[cfg(test)]
mod parser_tests {
    use xml::Parser;
    use xml::{StartTag, EndTag, PI, Comment, CDATA, Characters};

    #[test]
    fn test_start_tag() {
        let mut p = Parser::new();
        let mut i = 0;
        p.parse_str("<a>", |event| {
            i += 1;
            assert_eq!(event, Ok(StartTag(StartTag {
                name: "a".to_strbuf(),
                ns: None,
                prefix:None,
                attributes: Vec::new()
            })));
        });
        assert_eq!(i, 1);
    }

    #[test]
    fn test_end_tag() {
        let mut p = Parser::new();
        let mut i = 0;
        p.parse_str("</a>", |event| {
            i += 1;
            assert_eq!(event, Ok(EndTag(EndTag { name: "a".to_strbuf(), ns: None, prefix: None })));
        });
        assert_eq!(i, 1);
    }

    #[test]
    fn test_PI() {
        let mut p = Parser::new();
        let mut i = 0;
        p.parse_str("<?xml version='1.0' encoding='utf-8'?>", |event| {
            i += 1;
            assert_eq!(event, Ok(PI("xml version='1.0' encoding='utf-8'".to_strbuf())));
        });
        assert_eq!(i, 1);
    }

    #[test]
    fn test_comment() {
        let mut p = Parser::new();
        let mut i = 0;
        p.parse_str("<!--Nothing to see-->", |event| {
            i += 1;
            assert_eq!(event, Ok(Comment("Nothing to see".to_strbuf())));
        });
        assert_eq!(i, 1);
    }
    #[test]
    fn test_CDATA() {
        let mut p = Parser::new();
        let mut i = 0;
        p.parse_str("<![CDATA[<html><head><title>x</title></head><body/></html>]]>", |event| {
            i += 1;
            assert_eq!(event,
                       Ok(CDATA("<html><head><title>x</title></head><body/></html>".to_strbuf())));
        });
        assert_eq!(i, 1);
    }

    #[test]
    fn test_characters() {
        let mut p = Parser::new();
        let mut i = 0;
        p.parse_str("<text>Hello World, it&apos;s a nice day</text>", |event| {
            i += 1;
            if i == 2 {
                assert_eq!(event, Ok(Characters("Hello World, it's a nice day".to_strbuf())));
            }
        });
        assert_eq!(i, 3);
    }

    #[test]
    fn test_doctype() {
        let mut p = Parser::new();
        let mut i = 0;
        p.parse_str("<!DOCTYPE html>", |_| {
            i += 1;
        });
        assert_eq!(i, 0);
    }
}
