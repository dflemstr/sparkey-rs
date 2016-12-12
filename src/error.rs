use std::io;
use std::path;

use snap;

error_chain! {
    foreign_links {
        IO(io::Error);
        Snappy(snap::Error);
    }

    errors {
        PathNotUTF8(path: path::PathBuf) {}
        PathContainsNul(path: path::PathBuf, position: usize) {}

        VlqOverflow {}
        VlqUnderrun {}

        WrongLogMagicNumber {}
        WrongLogMajorVersion {}
        UnsupportedLogMinorVersion {}
        LogTooSmall {}
        LogIteratorInactive {}
        LogIteratorMismatch {}
        LogIteratorClosed {}
        LogHeaderCorrupt {}
        InvalidCompressionBlockSize {}
        InvalidCompressionType {}

        WrongHashMagicNumber {}
        WrongHashMajorVersion {}
        UnsupportedHashMinorVersion {}
        HashTooSmall {}
        FileIdentifierMismatch {}
        HashHeaderCorrupt {}
        HashSizeInvalid {}
        AddressSizeInvalid {}
    }
}
