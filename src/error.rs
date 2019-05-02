use std::io;
use std::path;
use std::result;

pub type Result<A> = result::Result<A, Error>;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "IO error")]
    IO(#[cause] io::Error),
    #[fail(display = "path not UTF-8: {:?}", path)]
    PathNotUTF8 { path: path::PathBuf },
    #[fail(
        display = "path contains null byte at position {}: {:?}",
        position, path
    )]
    PathContainsNul {
        path: path::PathBuf,
        position: usize,
    },

    #[fail(display = "internal error")]
    Internal,
    #[fail(display = "unexpected end-of-file")]
    UnexpectedEof,
    #[fail(display = "failed to mmap()")]
    MmapFailed,

    #[fail(display = "wrong log magic number")]
    WrongLogMagicNumber,
    #[fail(display = "wrong log major version")]
    WrongLogMajorVersion,
    #[fail(display = "unsupported log minor version")]
    UnsupportedLogMinorVersion,
    #[fail(display = "log too small")]
    LogTooSmall,
    #[fail(display = "log closed")]
    LogClosed,
    #[fail(display = "log iterator inactive")]
    LogIteratorInactive,
    #[fail(display = "log iterator mismatch")]
    LogIteratorMismatch,
    #[fail(display = "log iterator closed")]
    LogIteratorClosed,
    #[fail(display = "log header corrupt")]
    LogHeaderCorrupt,
    #[fail(display = "invalid compression block size")]
    InvalidCompressionBlockSize,
    #[fail(display = "invalid compression type")]
    InvalidCompressionType,

    #[fail(display = "wrong hash magic number")]
    WrongHashMagicNumber,
    #[fail(display = "wrong hash major version")]
    WrongHashMajorVersion,
    #[fail(display = "unsupported hash minor version")]
    UnsupportedHashMinorVersion,
    #[fail(display = "hash too small")]
    HashTooSmall,
    #[fail(display = "hash closed")]
    HashClosed,
    #[fail(display = "file identifier mismatch")]
    FileIdentifierMismatch,
    #[fail(display = "hash header corrupt")]
    HashHeaderCorrupt,
    #[fail(display = "hash size invalid")]
    HashSizeInvalid,
}
