// RustyXML
// Copyright (c) 2013 Florian Zeitz
//
// This project is MIT licensed.
// Please see the COPYING file for more information.

extern mod xml;

fn main()
{
    let mut p = xml::Parser::new();
    let mut e = xml::ElementBuilder::new();

    let stdin = std::io::stdin();
    loop {
        if stdin.eof() { return; }
        let input = stdin.read_line();
        do p.parse_str(input + "\n") |event| {
            /*
            match event {
                Ok(xml::PI(cont)) => print(format!("<?{}?>", cont)),
                Ok(xml::StartTag{name, attributes}) => {
                    print("<" + name);
                    for attr in attributes.iter() {
                        print(format!(" {}='{}'", attr.name, attr.value));
                    }
                    print(">");
                }
                Ok(xml::EndTag{name}) => print(format!("</{}>", name)),
                Ok(xml::Characters(chars)) => print(chars),
                Ok(xml::CDATA(chars)) => print(format!("<![CDATA[{}]]>", chars)),
                Ok(xml::Comment(cont)) => print(format!("<!--{}-->", cont)),
                // Ok(event) => println(format!("{}", event)),
                Err(e) => println(format!("Line: {} Column: {} Msg: {}", e.line, e.col, *e.msg)),
            }
            /*/
            match event {
                Ok(event) => match e.push_event(event) {
                    Ok(Some(e)) => println(e.to_str()),
                    Ok(None) => (),
                    Err(e) => println(format!("{}", e)),
                },
                Err(e) => println(format!("Line: {} Column: {} Msg: {}", e.line, e.col, *e.msg)),
            }
            //*/
        }
    }
}
