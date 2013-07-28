extern mod xml;

fn main()
{
    let mut p = xml::Parser();
    p.push_str(
"<?xml version='1.0'?>
<!DOCTYPE rand>
<hello type='greeting'>
  <![CDATA[some []]] stuff]]>
  <world size='big' location=\"milkyway\"/>
  foobar
</hello >
<!-- comment -->"
    );
    loop {
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
            //Ok(event) => println(fmt!("%?", event)),
            Err(e) => { println(fmt!("%?", e)); break; }
        }
    }
}
