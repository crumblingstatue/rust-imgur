#![feature(result_expect)]

extern crate imgur;

use std::fs::File;
use std::io::Read;

fn main() {
    let ref mut args = std::env::args();
    let filename = args.skip(1).next().expect("Need an image path as 1st argument");
    let id = args.next().expect("Need a client ID as 2nd argument");
    let mut file = File::open(filename).expect("Could not open image file");
    let mut data = Vec::new();
    file.read_to_end(&mut data).expect("Could not read image file");
    let handle = imgur::Handle::new(id);

    match handle.upload(&data) {
        Ok(info) => {
            match info.link() {
                Some(link) => println!("Success! Uploaded to {}", link),
                None => println!("Uploaded, but no link? Wat."),
            }
        }
        Err(e) => {
            println!("Error uploading: {}", e);
        }
    }
}
