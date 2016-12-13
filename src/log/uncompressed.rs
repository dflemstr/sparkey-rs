use std::borrow;
use std::io;

use error;
use log;
use util;

#[derive(Debug)]
pub struct EntryReader<'a> {
    reader: &'a log::Reader,
    next_position: usize,
}

impl<'a> EntryReader<'a> {
    pub fn new(reader: &log::Reader) -> EntryReader {
        EntryReader::new_at(reader, reader.header.header_size as usize)
    }

    pub fn new_at(reader: &log::Reader, position: usize) -> EntryReader {
        EntryReader {
            reader: reader,
            next_position: position,
        }
    }
}

impl<'a> log::EntryReader<'a> for EntryReader<'a> {
    fn next(&mut self)
            -> error::Result<Option<log::Entry<borrow::Cow<'a, [u8]>>>> {
        if self.next_position >= self.reader.header.data_end as usize {
            Ok(None)
        } else {
            let slice =
                unsafe { &self.reader.map.as_slice()[self.next_position..] };
            let mut cursor = io::Cursor::new(slice);
            let a = util::read_vlq(&mut cursor)? as usize;
            let b = util::read_vlq(&mut cursor)? as usize;
            let l = cursor.position() as usize;

            let entry = if a == 0 {
                let key = &slice[l..l + b];
                self.next_position += l + b;
                log::Entry::Delete(borrow::Cow::from(key))
            } else {
                let key = &slice[l..l + a - 1];
                let value = &slice[l + a - 1..l + a + b - 1];
                self.next_position += l + a + b - 1;
                log::Entry::Put(borrow::Cow::from(key),
                                borrow::Cow::from(value))
            };

            Ok(Some(entry))
        }
    }

    fn skip_next(&mut self) -> error::Result<()> {
        let slice =
            unsafe { &self.reader.map.as_slice()[self.next_position..] };
        let mut cursor = io::Cursor::new(slice);
        let a = util::read_vlq(&mut cursor)? as usize;
        let b = util::read_vlq(&mut cursor)? as usize;
        let l = cursor.position() as usize;

        if a == 0 {
            self.next_position += l + b;
        } else {
            self.next_position += l + a + b - 1;
        }

        Ok(())
    }
}
