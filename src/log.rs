use std::path;
use std::os;
use std::ptr;

use sparkey_sys::*;

use error;
use util;

pub enum CompressionType {
    None,
    Snappy,
}

pub enum EntryType {
    Put,
    Delete,
}

pub struct Writer(*mut logwriter);

pub struct Reader(*mut logreader, bool);

pub struct Entry {
    pub entry_type: EntryType,
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

pub struct Iter<'a>(*mut logiter, &'a Reader, Option<*mut hashreader>);

pub struct Keys<'a>(*mut logiter, &'a Reader, Option<*mut hashreader>);

pub struct Values<'a>(*mut logiter, &'a Reader, Option<*mut hashreader>);

impl CompressionType {
    pub fn from_raw(raw: compression_type) -> CompressionType {
        match raw {
            compression_type::COMPRESSION_NONE => CompressionType::None,
            compression_type::COMPRESSION_SNAPPY => CompressionType::Snappy,
        }
    }

    pub fn as_raw(&self) -> compression_type {
        match *self {
            CompressionType::None => compression_type::COMPRESSION_NONE,
            CompressionType::Snappy => compression_type::COMPRESSION_SNAPPY,
        }
    }
}

impl EntryType {
    pub fn from_raw(raw: entry_type) -> EntryType {
        match raw {
            entry_type::ENTRY_PUT => EntryType::Put,
            entry_type::ENTRY_DELETE => EntryType::Delete,
        }
    }

    pub fn as_raw(&self) -> entry_type {
        match *self {
            EntryType::Put => entry_type::ENTRY_PUT,
            EntryType::Delete => entry_type::ENTRY_DELETE,
        }
    }
}

impl Writer {
    pub fn create<P>(path: P,
                     compression_type: CompressionType,
                     compression_block_size: u32)
                     -> error::Result<Writer>
        where P: AsRef<path::Path>
    {
        let mut raw = ptr::null_mut();
        let path = util::path_to_cstring(path)?;

        util::handle(unsafe {
            logwriter_create(&mut raw,
                             path.as_ptr(),
                             compression_type.as_raw(),
                             compression_block_size as os::raw::c_int)
        })?;

        Ok(Writer(raw))
    }

    pub fn append<P>(path: P) -> error::Result<Writer>
        where P: AsRef<path::Path>
    {
        let mut raw = ptr::null_mut();
        let path = util::path_to_cstring(path)?;

        util::handle(unsafe { logwriter_append(&mut raw, path.as_ptr()) })?;

        Ok(Writer(raw))
    }

    pub unsafe fn from_raw(raw: *mut logwriter) -> Writer {
        Writer(raw)
    }

    pub fn as_raw(&self) -> *mut logwriter {
        self.0
    }

    pub fn put(&mut self, key: &[u8], value: &[u8]) -> error::Result<()> {
        util::handle(unsafe {
            logwriter_put(self.0,
                          key.len() as u64,
                          key.as_ptr(),
                          value.len() as u64,
                          value.as_ptr())
        })
    }

    pub fn delete(&mut self, key: &[u8]) -> error::Result<()> {
        util::handle(unsafe {
            logwriter_delete(self.0, key.len() as u64, key.as_ptr())
        })
    }

    pub fn flush(&mut self) -> error::Result<()> {
        util::handle(unsafe {
            logwriter_flush(self.0)
        })
    }
}

impl Drop for Writer {
    fn drop(&mut self) {
        util::handle(unsafe { logwriter_close(&mut self.0) }).unwrap()
    }
}

unsafe impl Send for Writer {}

impl Reader {
    pub fn open<P>(path: P) -> error::Result<Reader>
        where P: AsRef<path::Path>
    {
        let mut raw = ptr::null_mut();
        let path = util::path_to_cstring(path)?;

        util::handle(unsafe { logreader_open(&mut raw, path.as_ptr()) })?;

        Ok(Reader(raw, true))
    }

    pub unsafe fn from_raw(raw: *mut logreader) -> Reader {
        Reader(raw, false)
    }

    pub fn as_raw(&self) -> *mut logreader {
        self.0
    }

    pub fn max_key_len(&self) -> u64 {
        unsafe { logreader_maxkeylen(self.0) }
    }

    pub fn max_value_len(&self) -> u64 {
        unsafe { logreader_maxvaluelen(self.0) }
    }

    pub fn compression_block_size(&self) -> u32 {
        unsafe { logreader_get_compression_blocksize(self.0) as u32 }
    }

    pub fn compression_type(&self) -> CompressionType {
        unsafe {
            CompressionType::from_raw(logreader_get_compression_type(self.0))
        }
    }

    pub fn iter(&self) -> error::Result<Iter> {
        let mut raw = ptr::null_mut();

        util::handle(unsafe { logiter_create(&mut raw, self.0) })?;

        Ok(Iter(raw, self, None))
    }

    pub fn keys(&self) -> error::Result<Keys> {
        let mut raw = ptr::null_mut();

        util::handle(unsafe { logiter_create(&mut raw, self.0) })?;

        Ok(Keys(raw, self, None))
    }

    pub fn values(&self) -> error::Result<Values> {
        let mut raw = ptr::null_mut();

        util::handle(unsafe { logiter_create(&mut raw, self.0) })?;

        Ok(Values(raw, self, None))
    }
}

impl Drop for Reader {
    fn drop(&mut self) {
        if self.1 {
            unsafe { logreader_close(&mut self.0) }
        }
    }
}

unsafe impl Send for Reader {}

unsafe impl Sync for Reader {}

impl<'a> Iter<'a> {
    pub unsafe fn from_raw(raw: *mut logiter,
                           reader: &'a Reader,
                           hash: Option<*mut hashreader>)
                           -> Iter<'a> {
        Iter(raw, reader, hash)
    }

    pub fn as_raw(&self) -> *mut logiter {
        self.0
    }

    pub fn skip(&mut self, count: u32) -> error::Result<()> {
        util::handle(unsafe {
            logiter_skip(self.0, (self.1).0, count as os::raw::c_int)
        })
    }

    fn try_next(&mut self) -> error::Result<Option<Entry>> {
        if let Some(hash) = self.2 {
            util::handle(unsafe { logiter_hashnext(self.0, hash) })?;
        } else {
            util::handle(unsafe { logiter_next(self.0, (self.1).0) })?;
        }

        match unsafe { logiter_state(self.0) } {
            iter_state::ITER_ACTIVE => {
                let entry_type =
                    EntryType::from_raw(unsafe { logiter_type(self.0) });
                let key = util::read_key(self.0, (self.1).0)?;
                let value = util::read_value(self.0, (self.1).0)?;

                Ok(Some(Entry {
                    entry_type: entry_type,
                    key: key,
                    value: value,
                }))
            }
            _ => Ok(None),
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = error::Result<Entry>;

    fn next(&mut self) -> Option<Self::Item> {
        util::flip_option(self.try_next())
    }
}

impl<'a> Drop for Iter<'a> {
    fn drop(&mut self) {
        unsafe { logiter_close(&mut self.0) }
    }
}

unsafe impl<'a> Send for Iter<'a> {}

impl<'a> Keys<'a> {
    pub unsafe fn from_raw(raw: *mut logiter,
                           reader: &'a Reader,
                           hash: Option<*mut hashreader>)
                           -> Keys<'a> {
        Keys(raw, reader, hash)
    }

    pub fn as_raw(&self) -> *mut logiter {
        self.0
    }

    pub fn skip(&mut self, count: u32) -> error::Result<()> {
        util::handle(unsafe {
            logiter_skip(self.0, (self.1).0, count as os::raw::c_int)
        })
    }

    fn try_next(&mut self) -> error::Result<Option<Vec<u8>>> {
        if let Some(hash) = self.2 {
            util::handle(unsafe { logiter_hashnext(self.0, hash) })?;
        } else {
            util::handle(unsafe { logiter_next(self.0, (self.1).0) })?;
        }

        match unsafe { logiter_state(self.0) } {
            iter_state::ITER_ACTIVE => {
                let key = util::read_key(self.0, (self.1).0)?;

                Ok(Some(key))
            }
            _ => Ok(None),
        }
    }
}

impl<'a> Iterator for Keys<'a> {
    type Item = error::Result<Vec<u8>>;

    fn next(&mut self) -> Option<Self::Item> {
        util::flip_option(self.try_next())
    }
}

impl<'a> Drop for Keys<'a> {
    fn drop(&mut self) {
        unsafe { logiter_close(&mut self.0) }
    }
}

unsafe impl<'a> Send for Keys<'a> {}

impl<'a> Values<'a> {
    pub unsafe fn from_raw(raw: *mut logiter,
                           reader: &'a Reader,
                           hash: Option<*mut hashreader>)
                           -> Values<'a> {
        Values(raw, reader, hash)
    }

    pub fn as_raw(&self) -> *mut logiter {
        self.0
    }

    pub fn skip(&mut self, count: u32) -> error::Result<()> {
        util::handle(unsafe {
            logiter_skip(self.0, (self.1).0, count as os::raw::c_int)
        })
    }

    fn try_next(&mut self) -> error::Result<Option<Vec<u8>>> {
        if let Some(hash) = self.2 {
            util::handle(unsafe { logiter_hashnext(self.0, hash) })?;
        } else {
            util::handle(unsafe { logiter_next(self.0, (self.1).0) })?;
        }

        match unsafe { logiter_state(self.0) } {
            iter_state::ITER_ACTIVE => {
                let value = util::read_value(self.0, (self.1).0)?;

                Ok(Some(value))
            }
            _ => Ok(None),
        }
    }
}

impl<'a> Iterator for Values<'a> {
    type Item = error::Result<Vec<u8>>;

    fn next(&mut self) -> Option<Self::Item> {
        util::flip_option(self.try_next())
    }
}

impl<'a> Drop for Values<'a> {
    fn drop(&mut self) {
        unsafe { logiter_close(&mut self.0) }
    }
}

unsafe impl<'a> Send for Values<'a> {}
