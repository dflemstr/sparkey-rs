use std::ffi;
use std::io;
use std::os;
use std::path;
use std::result;

use libc;
use sparkey_sys::*;

use error;

pub fn path_to_cstring<P>(path: P) -> error::Result<ffi::CString>
    where P: AsRef<path::Path>
{
    use error::{Error, ErrorKind};
    let path = path.as_ref();
    let path_str = path.to_str()
        .ok_or(Error::from(ErrorKind::PathNotUTF8(path.to_path_buf())))?;

    match ffi::CString::new(path_str) {
        Ok(s) => Ok(s),
        Err(e) => {
            Err(error::ErrorKind::PathContainsNul(path.to_path_buf(),
                                                  e.nul_position())
                .into())
        }
    }
}

pub fn handle(returncode: returncode) -> error::Result<()> {
    use sparkey_sys::returncode::*;
    use error::ErrorKind::*;
    use error::ResultExt;

    fn err(kind: error::ErrorKind) -> error::Error {
        kind.into()
    }

    fn raw(raw: os::raw::c_int) -> error::Error {
        io::Error::from_raw_os_error(raw).into()
    }

    match returncode {
        SUCCESS => Ok(()),

        INTERNAL_ERROR => Err(err(InternalError)),

        FILE_NOT_FOUND => {
            Err(raw(libc::ENOENT)).chain_err(|| err(FileNotFound))
        }
        PERMISSION_DENIED => {
            Err(raw(libc::EACCES)).chain_err(|| err(PermissionDenied))
        }
        TOO_MANY_OPEN_FILES => {
            Err(raw(libc::ENFILE)).chain_err(|| err(TooManyOpenFiles))
        }
        FILE_TOO_LARGE => {
            Err(raw(libc::EOVERFLOW)).chain_err(|| err(FileTooLarge))
        }
        FILE_ALREADY_EXISTS => {
            Err(raw(libc::EEXIST)).chain_err(|| err(FileAlreadyExists))
        }
        FILE_BUSY => Err(raw(libc::EBUSY)).chain_err(|| err(FileBusy)),
        FILE_IS_DIRECTORY => {
            Err(raw(libc::EISDIR)).chain_err(|| err(FileIsDirectory))
        }
        FILE_SIZE_EXCEEDED => {
            Err(raw(libc::EFBIG)).chain_err(|| err(FileSizeExceeded))
        }
        FILE_CLOSED => Err(raw(libc::EBADF)).chain_err(|| err(FileClosed)),
        OUT_OF_DISK => Err(raw(libc::ENOSPC)).chain_err(|| err(OutOfDisk)),
        UNEXPECTED_EOF => {
            Err(raw(libc::ENOENT)).chain_err(|| err(FileNotFound))
        }
        MMAP_FAILED => Err(err(MmapFailed)),

        WRONG_LOG_MAGIC_NUMBER => Err(err(WrongLogMagicNumber)),
        WRONG_LOG_MAJOR_VERSION => Err(err(WrongLogMajorVersion)),
        UNSUPPORTED_LOG_MINOR_VERSION => Err(err(UnsupportedLogMinorVersion)),
        LOG_TOO_SMALL => Err(err(LogTooSmall)),
        LOG_CLOSED => Err(err(LogClosed)),
        LOG_ITERATOR_INACTIVE => Err(err(LogIteratorInactive)),
        LOG_ITERATOR_MISMATCH => Err(err(LogIteratorMismatch)),
        LOG_ITERATOR_CLOSED => Err(err(LogIteratorClosed)),
        LOG_HEADER_CORRUPT => Err(err(LogHeaderCorrupt)),
        INVALID_COMPRESSION_BLOCK_SIZE => Err(err(InvalidCompressionBlockSize)),
        INVALID_COMPRESSION_TYPE => Err(err(InvalidCompressionType)),

        WRONG_HASH_MAGIC_NUMBER => Err(err(WrongHashMagicNumber)),
        WRONG_HASH_MAJOR_VERSION => Err(err(WrongHashMajorVersion)),
        UNSUPPORTED_HASH_MINOR_VERSION => Err(err(UnsupportedHashMinorVersion)),
        HASH_TOO_SMALL => Err(err(HashTooSmall)),
        HASH_CLOSED => Err(err(HashClosed)),
        FILE_IDENTIFIER_MISMATCH => Err(err(FileIdentifierMismatch)),
        HASH_HEADER_CORRUPT => Err(err(HashHeaderCorrupt)),
        HASH_SIZE_INVALID => Err(err(HashSizeInvalid)),
    }
}

pub fn read_key(iter: *mut logiter,
                reader: *mut logreader)
                -> error::Result<Vec<u8>> {
    let expected_len = unsafe { logiter_keylen(iter) };
    let mut actual_len = 0;
    let mut buf = Vec::with_capacity(expected_len as usize);

    unsafe {
        handle(logiter_fill_key(iter,
                                reader,
                                expected_len,
                                buf.as_mut_ptr(),
                                &mut actual_len))?;
        assert_eq!(expected_len, actual_len);
        buf.set_len(actual_len as usize);
    }

    Ok(buf)
}

pub fn read_value(iter: *mut logiter,
                  reader: *mut logreader)
                  -> error::Result<Vec<u8>> {
    let expected_len = unsafe { logiter_valuelen(iter) };
    let mut actual_len = 0;
    let mut buf = Vec::with_capacity(expected_len as usize);

    unsafe {
        handle(logiter_fill_value(iter,
                                  reader,
                                  expected_len,
                                  buf.as_mut_ptr(),
                                  &mut actual_len))?;
        assert_eq!(expected_len, actual_len);
        buf.set_len(actual_len as usize);
    }

    Ok(buf)
}


pub fn flip_option<A, E>(option: result::Result<Option<A>, E>)
                         -> Option<result::Result<A, E>> {
    match option {
        Ok(None) => None,
        Ok(Some(r)) => Some(Ok(r)),
        Err(e) => Some(Err(e)),
    }
}
