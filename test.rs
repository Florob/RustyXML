extern mod xml;

fn main()
{
    let mut p = xml::Parser();
    let mut e = xml::ElementBuilder();

    let stdin = std::io::stdin();
    loop {
        if stdin.eof() { return; }
        let input = stdin.read_line();
        p.push_str(input + "\n");
        loop {
            /*
            match p.parse() {
                Ok(xml::Null) => break,
                Ok(xml::PI(cont)) => print(fmt!("<?%s?>", cont)),
                Ok(xml::StartTag(name, attrs)) => {
                    print(fmt!("<%s", name));
                    for attrs.iter().advance |attr| {
                        print(fmt!(" %s='%s'", attr.name, attr.value));
                    }
                    print(">");
                }
                Ok(xml::EndTag(name)) => print(fmt!("</%s>", name)),
                Ok(xml::Characters(chars)) => print(chars),
                Ok(xml::CDATA(chars)) => print(fmt!("<![CDATA[%s]]>", chars)),
                Ok(xml::Comment(cont)) => print(fmt!("<!--%s-->", cont)),
                // Ok(event) => println(fmt!("%?", event)),
                Err(e) => { println(fmt!("%?", e)); break; }
            }
            /*/
            match p.parse() {
                Ok(xml::Null) => break,
                Ok(event) => match e.push_event(event) {
                    Ok(Some(e)) => { println(fmt!("%?", e)); break; }
                    Ok(None) => loop,
                    Err(e) => { println(fmt!("%?", e)); return; }
                },
                Err(e) => { println(fmt!("%?", e)); return; }
            }
	    //*/
        }
    }
}
