// RustyXML
// Copyright (c) 2013-2015 Florian Zeitz
//
// This project is MIT licensed.
// Please see the COPYING file for more information.

// These are unstable for now
#![feature(exit_status)]

extern crate xml;
use std::fs::File;
use std::io::Read;

fn main() {
    let mut args = std::env::args();
    let name = args.next().unwrap_or("xmldemo".to_string());
    let path = args.next();
    let path = if let Some(ref path) = path {
        path
    } else {
        println!("Usage: {} <file>", name);
        return;
    };
    let mut rdr = match File::open(path) {
        Ok(file) => file,
        Err(err) => {
            println!("Couldn't open file: {}", err);
            std::env::set_exit_status(1);
            return;
        }
    };

    let mut p = xml::Parser::new();
    let mut e = xml::ElementBuilder::new();

    let mut string = String::new();
    if let Err(err) = rdr.read_to_string(&mut string) {
        println!("Reading failed: {}", err);
        std::env::set_exit_status(1);
        return;
    };

    p.feed_str(&string);
    for event in p.filter_map(|x| e.handle_event(x)) {
        // println!("{:?}", event);
        match event {
            Ok(e) => println!("{}", e),
            Err(e) => println!("{}", e),
        }
    }
}
