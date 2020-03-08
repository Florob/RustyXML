// RustyXML
// Copyright 2013-2016 RustyXML developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate xml;
use std::fs::File;
use std::io::Read;

fn main() {
    let mut args = std::env::args();
    let name = args.next().unwrap_or_else(|| "roundtrip".to_string());
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
            std::process::exit(1);
        }
    };

    let mut p = xml::Parser::new();
    let mut e = xml::ElementBuilder::new();

    let mut string = String::new();
    if let Err(err) = rdr.read_to_string(&mut string) {
        println!("Reading failed: {}", err);
        std::process::exit(1);
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
