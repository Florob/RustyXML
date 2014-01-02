// RustyXML
// Copyright (c) 2013 Florian Zeitz
//
// This project is MIT licensed.
// Please see the COPYING file for more information.

extern mod xml;
use std::io::File;
use std::io::Reader;
use std::path::Path;

fn main()
{
    let args = std::os::args();
    if args.len() != 2 {
        println!("Usage: {} <file>", args[0]);
        return;
    }

    let f = &Path::new(args[1].clone());
    if !f.exists() {
        println!("File '{}' does not exist", args[1]);
        return;
    }
    let mut rdr = File::open(f).expect("Couldn't open file");

    let mut p = xml::Parser::new();
    let mut e = xml::ElementBuilder::new();

    while !rdr.eof() {
        let mut buf = [0u8, ..4096];
        let mut len = match rdr.read(buf.mut_slice_to(4093)) {
            None => 0,
            Some(i) => i
        };

        if !std::str::is_utf8(buf) {
            let mut pos = len-1;
            while std::str::utf8_char_width(buf[pos]) == 0 {
                pos -= 1
            }
            let width = std::str::utf8_char_width(buf[pos]);
            let missing = pos+width-len;
            rdr.read(buf.mut_slice(len, len+missing));
            len += missing;
        }

        let string = std::str::from_utf8(buf.slice_to(len));
        p.parse_str(string, |event| {
            match event {
                Ok(event) => match e.push_event(event) {
                    Ok(Some(e)) => println!("{}", e),
                    Ok(None) => (),
                    Err(e) => println!("{}", e),
                },
                Err(e) => println!("Line: {} Column: {} Msg: {}", e.line, e.col, e.msg),
            }
            //println!("{:?}", event);
        });
    }
}
