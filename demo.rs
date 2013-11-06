// RustyXML
// Copyright (c) 2013 Florian Zeitz
//
// This project is MIT licensed.
// Please see the COPYING file for more information.

extern mod xml;
use std::rt::io::File;
use std::rt::io::Reader;
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
        let mut buf = [0u8, 4096];
        rdr.read(buf);
        let string = std::str::from_utf8(buf);
        do p.parse_str(string) |event| {
            match event {
                Ok(event) => match e.push_event(event) {
                    Ok(Some(e)) => println(e.to_str()),
                    Ok(None) => (),
                    Err(e) => println!("{}", e),
                },
                Err(e) => println!("Line: {} Column: {} Msg: {}", e.line, e.col, e.msg),
            }
        }
    }
}
