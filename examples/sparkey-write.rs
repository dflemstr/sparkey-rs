/// Reads key,value pairs from stdin and writes them to the specified
/// .spi/.spl files (given via command line args)
extern crate sparkey;

use std::env;
use std::io;
use std::path;

fn main() {
    use std::io::BufRead;

    let mut args = env::args_os().skip(1);
    let index = args.next().expect("No index (first arg) specified");
    let log = args.next().expect("No log (second arg) specified");

    let mut writer =
        sparkey::log::Writer::create(path::Path::new(&log),
                                     sparkey::log::CompressionType::Snappy(1024))
            .expect("Can't create log file");

    let stdin = io::stdin();

    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let mut parts = line.splitn(2, ",");
        let key = parts.next().unwrap();
        let value = parts.next();

        if let Some(value) = value {
            writer.put(key.as_bytes(), value.as_bytes())
                .expect("Can't put log entry");
        } else {
            writer.delete(key.as_bytes()).expect("Can't delete log entry");
        }
    }

    writer.flush().unwrap();

    sparkey::hash::Writer::write(index, log, None)
        .expect("Can't create index file");
}
