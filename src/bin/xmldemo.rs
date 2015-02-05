// RustyXML
// Copyright (c) 2013, 2014 Florian Zeitz
//
// This project is MIT licensed.
// Please see the COPYING file for more information.

#![feature(slicing_syntax)]

// These are unstable for now
#![feature(core)]
#![feature(env)]
#![feature(io)]
#![feature(os)]
#![feature(path)]

extern crate xml;
use std::old_io::File;
use std::old_io::Reader;
use std::old_path::Path;

fn main() {
    let mut args = std::env::args();
    let name = args.next().and_then(|x| x.into_string().ok()).unwrap_or("xmldemo".to_string());
    let f = if let Some(path) = args.next() {
        // FIXME: Workaround for `File::new()` not accepting `std::path::Path` yet
        use std::os::unix::OsStringExt;
        Path::new(path.into_vec())
    } else {
        println!("Usage: {} <file>", name);
        return;
    };
    let mut rdr = match File::open(&f) {
        Ok(file) => file,
        Err(err) => {
            println!("Couldn't open file: {}", err);
            std::env::set_exit_status(1);
            return;
        }
    };

    let mut p = xml::Parser::new();
    let mut e = xml::ElementBuilder::new();

    let string = match rdr.read_to_string() {
        Ok(string) => string,
        Err(err) => {
            println!("Reading failed: {}", err);
            std::env::set_exit_status(1);
            return;
        }
    };

    p.feed_str(&string[]);
    for event in p {
        // println!("{:?}", event);
        match e.push_event(event) {
            Ok(Some(e)) => println!("{}", e),
            Ok(None) => (),
            Err(e) => println!("{}", e),
        }
    }
}
