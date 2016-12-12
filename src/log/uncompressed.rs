use std::borrow;
use std::io;

use error;
use log;
use util;

#[derive(Debug)]
pub struct EntryReader<'a> {
    log_reader: &'a log::Reader,
    position: usize,
}

impl<'a> EntryReader<'a> {
    pub fn new(log_reader: &'a log::Reader) -> EntryReader<'a> {
        EntryReader::new_at(log_reader, log_reader.header.header_size as usize)
    }

    pub fn new_at(log_reader: &'a log::Reader,
                  position: usize)
                  -> EntryReader<'a> {
        EntryReader {
            log_reader: log_reader,
            position: position,
        }
    }
}

impl<'a> log::EntryReader<'a> for EntryReader<'a> {
    fn next(&mut self)
            -> error::Result<Option<log::Entry<borrow::Cow<'a, [u8]>>>> {
        let slice = unsafe { &self.log_reader.map.as_slice()[self.position..] };
        let mut cursor = io::Cursor::new(slice);
        let a = util::read_vlq(&mut cursor)? as usize;
        let b = util::read_vlq(&mut cursor)? as usize;
        let l = cursor.position() as usize;

        let entry = if a == 0 {
            let key = &slice[l..l + b];
            self.position += l + b;
            log::Entry::Delete(borrow::Cow::from(key))
        } else {
            let key = &slice[l..l + a - 1];
            let value = &slice[l + a - 1..l + a + b - 1];
            self.position += l + a + b - 1;
            log::Entry::Put(borrow::Cow::from(key), borrow::Cow::from(value))
        };

        Ok(Some(entry))
    }

    fn skip_next(&mut self) -> error::Result<()> {
        let slice = unsafe { &self.log_reader.map.as_slice()[self.position..] };
        let mut cursor = io::Cursor::new(slice);
        let a = util::read_vlq(&mut cursor)? as usize;
        let b = util::read_vlq(&mut cursor)? as usize;
        let l = cursor.position() as usize;

        if a == 0 {
            self.position += l + b;
        } else {
            self.position += l + a + b - 1;
        }

        Ok(())
    }
}
