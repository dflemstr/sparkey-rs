/* automatically generated by rust-bindgen */

#![allow(dead_code,
         non_camel_case_types,
         non_upper_case_globals,
         non_snake_case)]
pub type int8_t = i8;
pub type int16_t = i16;
pub type int32_t = i32;
pub type int64_t = i64;
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type uint64_t = u64;
pub type int_least8_t = ::std::os::raw::c_char;
pub type int_least16_t = ::std::os::raw::c_short;
pub type int_least32_t = ::std::os::raw::c_int;
pub type int_least64_t = ::std::os::raw::c_long;
pub type uint_least8_t = ::std::os::raw::c_uchar;
pub type uint_least16_t = ::std::os::raw::c_ushort;
pub type uint_least32_t = ::std::os::raw::c_uint;
pub type uint_least64_t = ::std::os::raw::c_ulong;
pub type int_fast8_t = ::std::os::raw::c_char;
pub type int_fast16_t = ::std::os::raw::c_long;
pub type int_fast32_t = ::std::os::raw::c_long;
pub type int_fast64_t = ::std::os::raw::c_long;
pub type uint_fast8_t = ::std::os::raw::c_uchar;
pub type uint_fast16_t = ::std::os::raw::c_ulong;
pub type uint_fast32_t = ::std::os::raw::c_ulong;
pub type uint_fast64_t = ::std::os::raw::c_ulong;
pub type intptr_t = isize;
pub type uintptr_t = usize;
pub type intmax_t = ::std::os::raw::c_long;
pub type uintmax_t = ::std::os::raw::c_ulong;
#[derive(Copy, Clone)]
#[repr(i32)]
#[derive(Debug)]
pub enum returncode {
    SUCCESS = 0,
    INTERNAL_ERROR = -1,
    FILE_NOT_FOUND = -100,
    PERMISSION_DENIED = -101,
    TOO_MANY_OPEN_FILES = -102,
    FILE_TOO_LARGE = -103,
    FILE_ALREADY_EXISTS = -104,
    FILE_BUSY = -105,
    FILE_IS_DIRECTORY = -106,
    FILE_SIZE_EXCEEDED = -107,
    FILE_CLOSED = -108,
    OUT_OF_DISK = -109,
    UNEXPECTED_EOF = -110,
    MMAP_FAILED = -111,
    WRONG_LOG_MAGIC_NUMBER = -200,
    WRONG_LOG_MAJOR_VERSION = -201,
    UNSUPPORTED_LOG_MINOR_VERSION = -202,
    LOG_TOO_SMALL = -203,
    LOG_CLOSED = -204,
    LOG_ITERATOR_INACTIVE = -205,
    LOG_ITERATOR_MISMATCH = -206,
    LOG_ITERATOR_CLOSED = -207,
    LOG_HEADER_CORRUPT = -208,
    INVALID_COMPRESSION_BLOCK_SIZE = -209,
    INVALID_COMPRESSION_TYPE = -210,
    WRONG_HASH_MAGIC_NUMBER = -300,
    WRONG_HASH_MAJOR_VERSION = -301,
    UNSUPPORTED_HASH_MINOR_VERSION = -302,
    HASH_TOO_SMALL = -303,
    HASH_CLOSED = -304,
    FILE_IDENTIFIER_MISMATCH = -305,
    HASH_HEADER_CORRUPT = -306,
    HASH_SIZE_INVALID = -307,
}
pub enum logwriter { }
#[derive(Copy, Clone)]
#[repr(u32)]
#[derive(Debug)]
pub enum compression_type { COMPRESSION_NONE = 0, COMPRESSION_SNAPPY = 1, }
#[derive(Copy, Clone)]
#[repr(u32)]
#[derive(Debug)]
pub enum entry_type { ENTRY_PUT = 0, ENTRY_DELETE = 1, }
#[derive(Copy, Clone)]
#[repr(u32)]
#[derive(Debug)]
pub enum iter_state {
    ITER_NEW = 0,
    ITER_ACTIVE = 1,
    ITER_CLOSED = 2,
    ITER_INVALID = 3,
}
pub enum logreader { }
pub enum logiter { }
pub enum hashreader { }
extern "C" {
    #[link_name = "sparkey_errstring"]
    pub fn errstring(code: returncode) -> *const ::std::os::raw::c_char;
    #[link_name = "sparkey_logwriter_create"]
    pub fn logwriter_create(log: *mut *mut logwriter,
                            filename: *const ::std::os::raw::c_char,
                            compression_type: compression_type,
                            compression_block_size: ::std::os::raw::c_int)
     -> returncode;
    #[link_name = "sparkey_logwriter_append"]
    pub fn logwriter_append(log: *mut *mut logwriter,
                            filename: *const ::std::os::raw::c_char)
     -> returncode;
    #[link_name = "sparkey_logwriter_put"]
    pub fn logwriter_put(log: *mut logwriter, keylen: uint64_t,
                         key: *const uint8_t, valuelen: uint64_t,
                         value: *const uint8_t) -> returncode;
    #[link_name = "sparkey_logwriter_delete"]
    pub fn logwriter_delete(log: *mut logwriter, keylen: uint64_t,
                            key: *const uint8_t) -> returncode;
    #[link_name = "sparkey_logwriter_flush"]
    pub fn logwriter_flush(log: *mut logwriter) -> returncode;
    #[link_name = "sparkey_logwriter_close"]
    pub fn logwriter_close(log: *mut *mut logwriter) -> returncode;
    #[link_name = "sparkey_logreader_open"]
    pub fn logreader_open(log: *mut *mut logreader,
                          filename: *const ::std::os::raw::c_char)
     -> returncode;
    #[link_name = "sparkey_logreader_close"]
    pub fn logreader_close(log: *mut *mut logreader);
    #[link_name = "sparkey_logreader_maxkeylen"]
    pub fn logreader_maxkeylen(log: *mut logreader) -> uint64_t;
    #[link_name = "sparkey_logreader_maxvaluelen"]
    pub fn logreader_maxvaluelen(log: *mut logreader) -> uint64_t;
    #[link_name = "sparkey_logreader_get_compression_blocksize"]
    pub fn logreader_get_compression_blocksize(log: *mut logreader)
     -> ::std::os::raw::c_int;
    #[link_name = "sparkey_logreader_get_compression_type"]
    pub fn logreader_get_compression_type(log: *mut logreader)
     -> compression_type;
    #[link_name = "sparkey_logiter_create"]
    pub fn logiter_create(iter: *mut *mut logiter, log: *mut logreader)
     -> returncode;
    #[link_name = "sparkey_logiter_close"]
    pub fn logiter_close(iter: *mut *mut logiter);
    #[link_name = "sparkey_logiter_seek"]
    pub fn logiter_seek(iter: *mut logiter, log: *mut logreader,
                        position: uint64_t) -> returncode;
    #[link_name = "sparkey_logiter_skip"]
    pub fn logiter_skip(iter: *mut logiter, log: *mut logreader,
                        count: ::std::os::raw::c_int) -> returncode;
    #[link_name = "sparkey_logiter_next"]
    pub fn logiter_next(iter: *mut logiter, log: *mut logreader)
     -> returncode;
    #[link_name = "sparkey_logiter_reset"]
    pub fn logiter_reset(iter: *mut logiter, log: *mut logreader)
     -> returncode;
    #[link_name = "sparkey_logiter_keychunk"]
    pub fn logiter_keychunk(iter: *mut logiter, log: *mut logreader,
                            maxlen: uint64_t, res: *mut *mut uint8_t,
                            len: *mut uint64_t) -> returncode;
    #[link_name = "sparkey_logiter_valuechunk"]
    pub fn logiter_valuechunk(iter: *mut logiter, log: *mut logreader,
                              maxlen: uint64_t, res: *mut *mut uint8_t,
                              len: *mut uint64_t) -> returncode;
    #[link_name = "sparkey_logiter_fill_key"]
    pub fn logiter_fill_key(iter: *mut logiter, log: *mut logreader,
                            maxlen: uint64_t, buf: *mut uint8_t,
                            len: *mut uint64_t) -> returncode;
    #[link_name = "sparkey_logiter_fill_value"]
    pub fn logiter_fill_value(iter: *mut logiter, log: *mut logreader,
                              maxlen: uint64_t, buf: *mut uint8_t,
                              len: *mut uint64_t) -> returncode;
    #[link_name = "sparkey_logiter_keycmp"]
    pub fn logiter_keycmp(iter1: *mut logiter, iter2: *mut logiter,
                          log: *mut logreader,
                          res: *mut ::std::os::raw::c_int) -> returncode;
    #[link_name = "sparkey_logiter_state"]
    pub fn logiter_state(iter: *mut logiter) -> iter_state;
    #[link_name = "sparkey_logiter_type"]
    pub fn logiter_type(iter: *mut logiter) -> entry_type;
    #[link_name = "sparkey_logiter_keylen"]
    pub fn logiter_keylen(iter: *mut logiter) -> uint64_t;
    #[link_name = "sparkey_logiter_valuelen"]
    pub fn logiter_valuelen(iter: *mut logiter) -> uint64_t;
    #[link_name = "sparkey_hash_write"]
    pub fn hash_write(hash_filename: *const ::std::os::raw::c_char,
                      log_filename: *const ::std::os::raw::c_char,
                      hash_size: ::std::os::raw::c_int) -> returncode;
    #[link_name = "sparkey_hash_open"]
    pub fn hash_open(reader: *mut *mut hashreader,
                     hash_filename: *const ::std::os::raw::c_char,
                     log_filename: *const ::std::os::raw::c_char)
     -> returncode;
    #[link_name = "sparkey_hash_getreader"]
    pub fn hash_getreader(reader: *mut hashreader) -> *mut logreader;
    #[link_name = "sparkey_hash_close"]
    pub fn hash_close(reader: *mut *mut hashreader);
    #[link_name = "sparkey_hash_get"]
    pub fn hash_get(reader: *mut hashreader, key: *const uint8_t,
                    keylen: uint64_t, iter: *mut logiter) -> returncode;
    #[link_name = "sparkey_logiter_hashnext"]
    pub fn logiter_hashnext(iter: *mut logiter, reader: *mut hashreader)
     -> returncode;
    #[link_name = "sparkey_hash_numentries"]
    pub fn hash_numentries(reader: *mut hashreader) -> uint64_t;
    #[link_name = "sparkey_hash_numcollisions"]
    pub fn hash_numcollisions(reader: *mut hashreader) -> uint64_t;
    #[link_name = "sparkey_create_log_filename"]
    pub fn create_log_filename(index_filename: *const ::std::os::raw::c_char)
     -> *mut ::std::os::raw::c_char;
    #[link_name = "sparkey_create_index_filename"]
    pub fn create_index_filename(log_filename: *const ::std::os::raw::c_char)
     -> *mut ::std::os::raw::c_char;
}
