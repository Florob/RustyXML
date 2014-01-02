extern mod xml;


#[cfg(test)]
mod base_tests {
    use xml::{escape, unescape, unrecognized_entity};
    use xml::{Element, Attribute, CharacterNode, CDATANode, CommentNode, PINode};
    use std::hashmap::HashMap;

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
    fn test_unescape_cond() {
        unrecognized_entity::cond.trap(|ent| {
            if ent.as_slice() == "&nbsp;" { ~"\u00a0" } else { ent }
        }).inside(|| {
            let unesc = unescape("&nbsp;&foo;");
            assert_eq!(unesc, ~"\u00a0&foo;");
        })
    }

    #[test]
    fn test_to_str_element() {
        let elem = Element {
            name: ~"a",
            ns: None,
            default_ns: None,
            prefixes: HashMap::new(),
            attributes: ~[],
            children: ~[]
        };
        assert_eq!(elem.to_str(), ~"<a/>");

        let elem = Element {
            name: ~"a",
            ns: None,
            default_ns: None,
            prefixes: HashMap::new(),
            attributes: ~[
                Attribute { name: ~"href", ns: None, value: ~"http://rust-lang.org" }
            ],
            children: ~[]
        };
        assert_eq!(elem.to_str(), ~"<a href='http://rust-lang.org'/>");

        let elem = Element {
            name: ~"a",
            ns: None,
            default_ns: None,
            prefixes: HashMap::new(),
            attributes: ~[],
            children: ~[
                Element(Element {
                    name: ~"b",
                    ns: None,
                    default_ns: None,
                    prefixes: HashMap::new(),
                    attributes: ~[],
                    children: ~[]
                })
            ]
        };
        assert_eq!(elem.to_str(), ~"<a><b/></a>");

        let elem = Element {
            name: ~"a",
            ns: None,
            default_ns: None,
            prefixes: HashMap::new(),
            attributes: ~[
                Attribute { name: ~"href", ns: None, value: ~"http://rust-lang.org" }
            ],
            children: ~[
                Element(Element {
                    name: ~"b",
                    ns: None,
                    default_ns: None,
                    prefixes: HashMap::new(),
                    attributes: ~[],
                    children: ~[]
                })
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
            ns: None,
            default_ns: None,
            prefixes: HashMap::new(),
            attributes: ~[],
            children: ~[
                PINode(~"processing information"),
                CDATANode(~"<hello/>"),
                Element(Element{
                    name: ~"b",
                    ns: None,
                    default_ns: None,
                    prefixes: HashMap::new(),
                    attributes: ~[],
                    children: ~[]
                }),
                CharacterNode(~"World"),
                CommentNode(~"Nothing to see")
            ]
        };
        assert_eq!(elem.content_str(), ~"<hello/>World");
    }
}

#[cfg(test)]
mod base_bench {
    extern mod extra;
    use self::extra::test::BenchHarness;
    use xml::{escape, unescape};

    #[bench]
    fn bench_escape(bh: &mut BenchHarness) {
        let input = "&<>'\"".repeat(100);
        bh.iter( || {
            escape(input);
        });
        bh.bytes = input.len() as u64;
    }

    #[bench]
    fn bench_unescape(bh: &mut BenchHarness) {
        let input = "&amp;&lt;&gt;&apos;&quot;".repeat(50);
        bh.iter(|| {
            unescape(input);
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
                name: ~"a",
                ns: None,
                prefix:None,
                attributes: ~[]
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
            assert_eq!(event, Ok(EndTag(EndTag { name: ~"a", ns: None, prefix: None })));
        });
        assert_eq!(i, 1);
    }

    #[test]
    fn test_PI() {
        let mut p = Parser::new();
        let mut i = 0;
        p.parse_str("<?xml version='1.0' encoding='utf-8'?>", |event| {
            i += 1;
            assert_eq!(event, Ok(PI(~"xml version='1.0' encoding='utf-8'")));
        });
        assert_eq!(i, 1);
    }

    #[test]
    fn test_comment() {
        let mut p = Parser::new();
        let mut i = 0;
        p.parse_str("<!--Nothing to see-->", |event| {
            i += 1;
            assert_eq!(event, Ok(Comment(~"Nothing to see")));
        });
        assert_eq!(i, 1);
    }
    #[test]
    fn test_CDATA() {
        let mut p = Parser::new();
        let mut i = 0;
        p.parse_str("<![CDATA[<html><head><title>x</title></head><body/></html>]]>", |event| {
            i += 1;
            assert_eq!(event, Ok(CDATA(~"<html><head><title>x</title></head><body/></html>")));
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
                assert_eq!(event, Ok(Characters(~"Hello World, it's a nice day")));
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
