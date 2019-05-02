use std::ffi;
use std::io;
use std::os;
use std::path;
use std::result;

use libc;
use sparkey_sys::*;

use error;

pub fn path_to_cstring<P>(path: P) -> error::Result<ffi::CString>
where
    P: AsRef<path::Path>,
{
    let path = path.as_ref();
    let path_str = path
        .to_str()
        .ok_or(error::Error::PathNotUTF8 { path: path.to_path_buf() })?;

    match ffi::CString::new(path_str) {
        Ok(s) => Ok(s),
        Err(e) => {
            Err(error::Error::PathContainsNul { path: path.to_path_buf(), position: e.nul_position() })
        }
    }
}

pub fn handle(returncode: returncode) -> error::Result<()> {
    use error::Error::*;
    use sparkey_sys::returncode::*;

    fn raw(raw: os::raw::c_int) -> error::Error {
        error::Error::IO(io::Error::from_raw_os_error(raw))
    }

    match returncode {
        SUCCESS => Ok(()),

        INTERNAL_ERROR => Err(InternalError),

        FILE_NOT_FOUND => Err(raw(libc::ENOENT)),
        PERMISSION_DENIED => Err(raw(libc::EACCES)),
        TOO_MANY_OPEN_FILES => Err(raw(libc::ENFILE)),
        FILE_TOO_LARGE => Err(raw(libc::EOVERFLOW)),
        FILE_ALREADY_EXISTS => Err(raw(libc::EEXIST)),
        FILE_BUSY => Err(raw(libc::EBUSY)),
        FILE_IS_DIRECTORY => Err(raw(libc::EISDIR)),
        FILE_SIZE_EXCEEDED => Err(raw(libc::EFBIG)),
        FILE_CLOSED => Err(raw(libc::EBADF)),
        OUT_OF_DISK => Err(raw(libc::ENOSPC)),
        UNEXPECTED_EOF => Err(raw(libc::ENOENT)),
        MMAP_FAILED => Err(MmapFailed),

        WRONG_LOG_MAGIC_NUMBER => Err(WrongLogMagicNumber),
        WRONG_LOG_MAJOR_VERSION => Err(WrongLogMajorVersion),
        UNSUPPORTED_LOG_MINOR_VERSION => Err(UnsupportedLogMinorVersion),
        LOG_TOO_SMALL => Err(LogTooSmall),
        LOG_CLOSED => Err(LogClosed),
        LOG_ITERATOR_INACTIVE => Err(LogIteratorInactive),
        LOG_ITERATOR_MISMATCH => Err(LogIteratorMismatch),
        LOG_ITERATOR_CLOSED => Err(LogIteratorClosed),
        LOG_HEADER_CORRUPT => Err(LogHeaderCorrupt),
        INVALID_COMPRESSION_BLOCK_SIZE => Err(InvalidCompressionBlockSize),
        INVALID_COMPRESSION_TYPE => Err(InvalidCompressionType),

        WRONG_HASH_MAGIC_NUMBER => Err(WrongHashMagicNumber),
        WRONG_HASH_MAJOR_VERSION => Err(WrongHashMajorVersion),
        UNSUPPORTED_HASH_MINOR_VERSION => Err(UnsupportedHashMinorVersion),
        HASH_TOO_SMALL => Err(HashTooSmall),
        HASH_CLOSED => Err(HashClosed),
        FILE_IDENTIFIER_MISMATCH => Err(FileIdentifierMismatch),
        HASH_HEADER_CORRUPT => Err(HashHeaderCorrupt),
        HASH_SIZE_INVALID => Err(HashSizeInvalid),
    }
}

pub fn read_key(iter: *mut logiter, reader: *mut logreader) -> error::Result<Vec<u8>> {
    let expected_len = unsafe { logiter_keylen(iter) };
    let mut actual_len = 0;
    let mut buf = Vec::with_capacity(expected_len as usize);

    unsafe {
        handle(logiter_fill_key(
            iter,
            reader,
            expected_len,
            buf.as_mut_ptr(),
            &mut actual_len,
        ))?;
        assert_eq!(expected_len, actual_len);
        buf.set_len(actual_len as usize);
    }

    Ok(buf)
}

pub fn read_value(iter: *mut logiter, reader: *mut logreader) -> error::Result<Vec<u8>> {
    let expected_len = unsafe { logiter_valuelen(iter) };
    let mut actual_len = 0;
    let mut buf = Vec::with_capacity(expected_len as usize);

    unsafe {
        handle(logiter_fill_value(
            iter,
            reader,
            expected_len,
            buf.as_mut_ptr(),
            &mut actual_len,
        ))?;
        assert_eq!(expected_len, actual_len);
        buf.set_len(actual_len as usize);
    }

    Ok(buf)
}

pub fn flip_option<A, E>(option: result::Result<Option<A>, E>) -> Option<result::Result<A, E>> {
    match option {
        Ok(None) => None,
        Ok(Some(r)) => Some(Ok(r)),
        Err(e) => Some(Err(e)),
    }
}
