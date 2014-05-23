// RustyXML
// Copyright (c) 2013, 2014 Florian Zeitz
//
// This project is MIT licensed.
// Please see the COPYING file for more information.

extern crate xml;
use std::io::File;
use std::io::Reader;
use std::path::Path;

fn main()
{
    let args = std::os::args();
    let f = &match args.as_slice() {
        [_, ref path] => Path::new(path.as_slice()),
        [ref name, ..] => {
            println!("Usage: {} <file>", name);
            return;
        }
        _ => fail!("argv had length 0")
    };
    let mut rdr = match File::open(f) {
        Ok(file) => file,
        Err(err) => {
            println!("Couldn't open file: {}", err);
            std::os::set_exit_status(1);
            return;
        }
    };

    let mut p = xml::Parser::new();
    let mut e = xml::ElementBuilder::new();

    let string = match rdr.read_to_str() {
        Ok(string) => string,
        Err(err) => {
            println!("Reading failed: {}", err);
            std::os::set_exit_status(1);
            return;
        }
    };

    p.parse_str(string.as_slice(), |event| {
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
