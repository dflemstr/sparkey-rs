use std::os;
use std::path;
use std::ptr;

use sparkey_sys::*;

use crate::error;
use crate::log;
use crate::util;

#[allow(non_camel_case_types)]
pub enum Type {
    Murmur3_32,
    Murmur3_64,
}

pub struct Writer;

pub struct Reader(*mut hashreader, log::Reader);

impl Type {
    fn as_raw(&self) -> os::raw::c_int {
        match *self {
            Type::Murmur3_32 => 4,
            Type::Murmur3_64 => 8,
        }
    }
}

impl Writer {
    pub fn write<P1, P2>(hash_path: P1, log_path: P2, hash_type: Option<Type>) -> error::Result<()>
    where
        P1: AsRef<path::Path>,
        P2: AsRef<path::Path>,
    {
        let hash_path = util::path_to_cstring(hash_path)?;
        let log_path = util::path_to_cstring(log_path)?;

        util::handle(unsafe {
            hash_write(
                hash_path.as_ptr(),
                log_path.as_ptr(),
                hash_type.map_or(0, |t| t.as_raw()),
            )
        })?;

        Ok(())
    }
}

impl Reader {
    pub fn open<P1, P2>(hash_path: P1, log_path: P2) -> error::Result<Self>
    where
        P1: AsRef<path::Path>,
        P2: AsRef<path::Path>,
    {
        let mut raw = ptr::null_mut();
        let hash_path = util::path_to_cstring(hash_path)?;
        let log_path = util::path_to_cstring(log_path)?;

        util::handle(unsafe { hash_open(&mut raw, hash_path.as_ptr(), log_path.as_ptr()) })?;

        let log_reader = unsafe { log::Reader::from_raw(hash_getreader(raw)) };

        Ok(Self(raw, log_reader))
    }

    pub fn log_reader(&self) -> &log::Reader {
        &self.1
    }

    pub fn get(&self, key: &[u8]) -> error::Result<Option<Vec<u8>>> {
        let log_reader = self.log_reader().as_raw();
        let mut log_iter = ptr::null_mut();

        util::handle(unsafe { logiter_create(&mut log_iter, log_reader) })?;

        util::handle(unsafe { hash_get(self.0, key.as_ptr(), key.len() as u64, log_iter) })?;

        let result = match unsafe { logiter_state(log_iter) } {
            iter_state::ITER_ACTIVE => {
                let value = util::read_value(log_iter, log_reader)?;
                Some(value)
            }
            _ => None,
        };

        unsafe { logiter_close(&mut log_iter) };

        Ok(result)
    }

    pub fn entries(&self) -> error::Result<log::Entries> {
        let mut raw = ptr::null_mut();

        util::handle(unsafe { logiter_create(&mut raw, self.1.as_raw()) })?;

        Ok(unsafe { log::Entries::from_raw(raw, &self.1, Some(self.0)) })
    }

    pub fn keys(&self) -> error::Result<log::Keys> {
        let mut raw = ptr::null_mut();

        util::handle(unsafe { logiter_create(&mut raw, self.1.as_raw()) })?;

        Ok(unsafe { log::Keys::from_raw(raw, &self.1, Some(self.0)) })
    }

    pub fn values(&self) -> error::Result<log::Values> {
        let mut raw = ptr::null_mut();

        util::handle(unsafe { logiter_create(&mut raw, self.1.as_raw()) })?;

        Ok(unsafe { log::Values::from_raw(raw, &self.1, Some(self.0)) })
    }

    pub fn num_entries(&self) -> u64 {
        unsafe { hash_numentries(self.0) }
    }

    pub fn num_collisions(&self) -> u64 {
        unsafe { hash_numcollisions(self.0) }
    }
}

impl Drop for Reader {
    fn drop(&mut self) {
        unsafe { hash_close(&mut self.0) }
    }
}

unsafe impl Send for Reader {}

unsafe impl Sync for Reader {}
