use std::io;
use std::path;

error_chain! {
    foreign_links {
        IO(io::Error);
    }

    errors {
        PathNotUTF8(path: path::PathBuf) {}
        PathContainsNul(path: path::PathBuf, position: usize) {}

        InternalError {}

        FileNotFound {}
        PermissionDenied {}
        TooManyOpenFiles {}
        FileTooLarge {}
        FileAlreadyExists {}
        FileBusy {}
        FileIsDirectory {}
        FileSizeExceeded {}
        FileClosed {}
        OutOfDisk {}
        UnexpectedEof {}
        MmapFailed {}

        WrongLogMagicNumber {}
        WrongLogMajorVersion {}
        UnsupportedLogMinorVersion {}
        LogTooSmall {}
        LogClosed {}
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
        HashClosed {}
        FileIdentifierMismatch {}
        HashHeaderCorrupt {}
        HashSizeInvalid {}
    }
}
