use openpty::openpty;
use std::io::{Read, Write};

fn main() {
    let (mut master, mut slave, _) = openpty(None, None, None).unwrap();

    // like a subprocess would, write to slave fd
    println!("I wrote Hello world!");
    slave.write_all(b"Hello world!").unwrap();
    drop(slave); // avoid potential deadlock with read below

    let mut out = String::new();
    // this will generate an I/O error because slave was closed
    master.read_to_string(&mut out).unwrap_err();
    println!("I read {}", out);
}
