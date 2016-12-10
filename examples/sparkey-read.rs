extern crate sparkey;

use std::env;
use std::path;
use std::str;

fn main() {
    let mut args = env::args_os().skip(1);
    let index = args.next().expect("No index (first arg) specified");
    let log = args.next().expect("No log (second arg) specified");

    let reader = sparkey::hash::Reader::open(path::Path::new(&index),
                                             path::Path::new(&log))
        .expect("Can't open files");

    for entry in reader.entries().unwrap() {
        let entry = entry.unwrap();
        println!("{},{}",
                 str::from_utf8(&entry.key).unwrap(),
                 str::from_utf8(&entry.value).unwrap());
    }
}
