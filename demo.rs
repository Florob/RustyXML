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
                Ok(xml::PI(cont)) => print!("<?{}?>", cont),
                Ok(xml::StartTag(xml::StartTag{name, attributes})) => {
                    print("<" + name);
                    for attr in attributes.iter() {
                        print!(" {}='{}'", attr.name, attr.value);
                    }
                    print(">");
                }
                Ok(xml::EndTag(xml::EndTag{name})) => print!("</{}>", name),
                Ok(xml::Characters(chars)) => print(chars),
                Ok(xml::CDATA(chars)) => print!("<![CDATA[{}]]>", chars),
                Ok(xml::Comment(cont)) => print!("<!--{}-->", cont),
                // Ok(event) => println!("{}", event),
                Err(e) => println!("Line: {} Column: {} Msg: {}", e.line, e.col, *e.msg),
            }
            /*/
            match event {
                Ok(event) => match e.push_event(event) {
                    Ok(Some(e)) => println(e.to_str()),
                    Ok(None) => (),
                    Err(e) => println!("{}", e),
                },
                Err(e) => println!("Line: {} Column: {} Msg: {}", e.line, e.col, *e.msg),
            }
            //*/
        }
    }
}
