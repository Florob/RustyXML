// RustyXML
// Copyright (c) 2013, 2014 Florian Zeitz
//
// This project is MIT licensed.
// Please see the COPYING file for more information.

#![feature(slicing_syntax)]

extern crate xml;
use std::io::File;
use std::io::Reader;
use std::path::Path;

fn main()
{
    let args = std::os::args();
    let f = &match args[] {
        [_, ref path] => Path::new(path[]),
        [ref name, ..] => {
            println!("Usage: {} <file>", name);
            return;
        }
        _ => panic!("argv had length 0")
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

    let string = match rdr.read_to_string() {
        Ok(string) => string,
        Err(err) => {
            println!("Reading failed: {}", err);
            std::os::set_exit_status(1);
            return;
        }
    };

    p.feed_str(string[]);
    for event in p {
        match event {
            Ok(event) => match e.push_event(event) {
                Ok(Some(e)) => println!("{}", e),
                Ok(None) => (),
                Err(e) => println!("{}", e),
            },
            Err(e) => println!("Line: {} Column: {} Msg: {}", e.line, e.col, e.msg),
        }
        // println!("{}", event);
    }
}
