extern crate xml;

#[cfg(test)]
mod base_tests {
    extern crate collections;

    use xml::{escape, unescape};
    use xml::{Element, Attribute, CharacterNode, CDATANode, CommentNode, PINode};

    #[test]
    fn test_escape() {
        let esc = escape("&<>'\"");
        assert_eq!(esc, "&amp;&lt;&gt;&apos;&quot;".to_string());
    }

    #[test]
    fn test_unescape() {
        let unesc = unescape("&amp;lt;&lt;&gt;&apos;&quot;&#x201c;&#x201d;&#38;&#34;");
        assert_eq!(unesc, Ok("&lt;<>'\"\u201c\u201d&\"".to_string()));
    }

    #[test]
    fn test_unescape_invalid() {
        let unesc = unescape("&amp;&nbsp;");
        assert_eq!(unesc, Err("&nbsp;".to_string()));
    }

    #[test]
    fn test_show_element() {
        let elem = Element::new("a", None, Vec::new());
        assert_eq!(format!("{}", elem).as_slice(), "<a/>");

        let elem = Element::new("a", None, vec!(
            Attribute {
                name: "href".to_string(),
                ns: None,
                value: "http://rust-lang.org".to_string()
            }
        ));
        assert_eq!(format!("{}", elem).as_slice(), "<a href='http://rust-lang.org'/>");

        let mut elem = Element::new("a", None, Vec::new());
        elem.tag(Element::new("b", None, Vec::new()));
        assert_eq!(format!("{}", elem).as_slice(), "<a><b/></a>");

        let mut elem = Element::new("a", None, vec!(
            Attribute {
                name: "href".to_string(),
                ns: None,
                value: "http://rust-lang.org".to_string()
            }
        ));
        elem.tag(Element::new("b", None, Vec::new()));
        assert_eq!(format!("{}", elem).as_slice(), "<a href='http://rust-lang.org'><b/></a>");
    }

    #[test]
    fn test_show_characters() {
        let chars = CharacterNode("some text".to_string());
        assert_eq!(format!("{}", chars).as_slice(), "some text");
    }

    #[test]
    fn test_show_CDATA() {
        let chars = CDATANode("some text".to_string());
        assert_eq!(format!("{}", chars).as_slice(), "<![CDATA[some text]]>");
    }

    #[test]
    fn test_show_comment() {
        let chars = CommentNode("some text".to_string());
        assert_eq!(format!("{}", chars).as_slice(), "<!--some text-->");
    }

    #[test]
    fn test_show_pi() {
        let chars = PINode("xml version='1.0'".to_string());
        assert_eq!(format!("{}", chars).as_slice(), "<?xml version='1.0'?>");
    }

    #[test]
    fn test_content_str() {
        let mut elem = Element::new("a", None, Vec::new());
        elem.pi("processing information")
            .cdata("<hello/>")
            .tag_stay(Element::new("b", None, Vec::new()))
            .text("World")
            .comment("Nothing to see");
        assert_eq!(elem.content_str(), "<hello/>World".to_string());
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
            escape(input.as_slice())
        });
        bh.bytes = input.len() as u64;
    }

    #[bench]
    fn bench_unescape(bh: &mut Bencher) {
        let input = "&amp;&lt;&gt;&apos;&quot;".repeat(50);
        bh.iter(|| {
            unescape(input.as_slice())
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
                name: "a".to_string(),
                ns: None,
                prefix: None,
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
            assert_eq!(event, Ok(EndTag(EndTag {
                name: "a".to_string(),
                ns: None,
                prefix: None
            })));
        });
        assert_eq!(i, 1);
    }

    #[test]
    fn test_self_closing_with_space() {
        let mut p = Parser::new();
        let mut v = Vec::new();
        p.parse_str("<register />", |event| {
            v.push(event);
        });
        assert_eq!(v, vec![
            Ok(StartTag(StartTag {
                name: "register".to_string(),
                ns: None,
                prefix: None,
                attributes: Vec::new()
            })),
            Ok(EndTag(EndTag {
                name: "register".to_string(),
                ns: None,
                prefix: None,
            }))
        ]);
    }

    #[test]
    fn test_self_closing_without_space() {
        let mut p = Parser::new();
        let mut v = Vec::new();
        p.parse_str("<register/>", |event| {
            v.push(event);
        });
        assert_eq!(v, vec![
            Ok(StartTag(StartTag {
                name: "register".to_string(),
                ns: None,
                prefix: None,
                attributes: Vec::new()
            })),
            Ok(EndTag(EndTag {
                name: "register".to_string(),
                ns: None,
                prefix: None,
            }))
        ]);
    }

    #[test]
    fn test_PI() {
        let mut p = Parser::new();
        let mut i = 0;
        p.parse_str("<?xml version='1.0' encoding='utf-8'?>", |event| {
            i += 1;
            assert_eq!(event, Ok(PI("xml version='1.0' encoding='utf-8'".to_string())));
        });
        assert_eq!(i, 1);
    }

    #[test]
    fn test_comment() {
        let mut p = Parser::new();
        let mut i = 0;
        p.parse_str("<!--Nothing to see-->", |event| {
            i += 1;
            assert_eq!(event, Ok(Comment("Nothing to see".to_string())));
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
                       Ok(CDATA("<html><head><title>x</title></head><body/></html>".to_string())));
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
                assert_eq!(event, Ok(Characters("Hello World, it's a nice day".to_string())));
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
