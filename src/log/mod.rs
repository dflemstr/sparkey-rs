use std::borrow;
use std::fmt;
use std::fs;
use std::io;
use std::path;

use byteorder;
use memmap;

use error;
use util;

mod snappy;
mod uncompressed;

const MAGIC: u32 = 0x49b39c95;
const MAJOR_VERSION: u32 = 1;
const MINOR_VERSION: u32 = 0;
const HEADER_SIZE: u32 = 84;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CompressionType {
    None,
    Snappy(u32),
}

#[derive(Debug)]
struct Header {
    major_version: u32,
    minor_version: u32,
    file_identifier: u32,
    num_puts: u64,
    num_deletes: u64,
    data_end: u64,
    max_key_len: u64,
    max_value_len: u64,
    delete_size: u64,
    compression_type: CompressionType,
    put_size: u64,
    header_size: u32,
    max_entries_per_block: u32,
}

#[derive(Debug)]
pub struct Writer {
    header: Header,
    write: io::BufWriter<fs::File>,
}

#[derive(Debug)]
pub struct Reader {
    header: Header,
    map: memmap::Mmap,
}

#[derive(Debug)]
pub enum Entry<A> {
    Put(A, A),
    Delete(A),
}

trait EntryReader<'a>: fmt::Debug {
    fn next(&mut self) -> error::Result<Option<Entry<borrow::Cow<'a, [u8]>>>>;

    fn skip_next(&mut self) -> error::Result<()>;
}

#[derive(Debug)]
pub struct Entries<'a>(Box<EntryReader<'a> + 'a>);

#[derive(Debug)]
pub struct OwnedEntries<'a>(Entries<'a>);

impl Header {
    fn load(path: &path::Path) -> error::Result<Header> {
        use byteorder::ReadBytesExt;

        let mut file = fs::File::open(path)?;

        let magic = file.read_u32::<byteorder::LittleEndian>()?;
        if magic != MAGIC {
            bail!(error::ErrorKind::WrongLogMagicNumber);
        }

        let major_version = file.read_u32::<byteorder::LittleEndian>()?;
        if major_version != MAJOR_VERSION {
            bail!(error::ErrorKind::WrongLogMajorVersion);
        }

        let minor_version = file.read_u32::<byteorder::LittleEndian>()?;
        if minor_version > MINOR_VERSION {
            bail!(error::ErrorKind::UnsupportedLogMinorVersion);
        }

        let file_identifier = file.read_u32::<byteorder::LittleEndian>()?;
        let num_puts = file.read_u64::<byteorder::LittleEndian>()?;
        let num_deletes = file.read_u64::<byteorder::LittleEndian>()?;
        let data_end = file.read_u64::<byteorder::LittleEndian>()?;
        let max_key_len = file.read_u64::<byteorder::LittleEndian>()?;
        let max_value_len = file.read_u64::<byteorder::LittleEndian>()?;
        let delete_size = file.read_u64::<byteorder::LittleEndian>()?;
        let compression_type = file.read_u32::<byteorder::LittleEndian>()?;
        let compression_block_size =
            file.read_u32::<byteorder::LittleEndian>()?;
        let put_size = file.read_u64::<byteorder::LittleEndian>()?;
        let max_entries_per_block = file.read_u32::<byteorder::LittleEndian>()?;

        let compression_type = match compression_type {
            0 => CompressionType::None,
            1 => CompressionType::Snappy(compression_block_size),
            _ => bail!(error::ErrorKind::InvalidCompressionType),
        };

        if data_end < HEADER_SIZE as u64 {
            bail!(error::ErrorKind::LogHeaderCorrupt);
        }

        if num_puts > data_end {
            bail!(error::ErrorKind::LogHeaderCorrupt);
        }

        if num_deletes > data_end {
            bail!(error::ErrorKind::LogHeaderCorrupt);
        }

        let header = Header {
            major_version: major_version,
            minor_version: minor_version,
            file_identifier: file_identifier,
            num_puts: num_puts,
            num_deletes: num_deletes,
            data_end: data_end,
            max_key_len: max_key_len,
            max_value_len: max_value_len,
            delete_size: delete_size,
            compression_type: compression_type,
            put_size: put_size,
            header_size: HEADER_SIZE,
            max_entries_per_block: max_entries_per_block,
        };

        Ok(header)
    }
}

impl Writer {
    pub fn create<P>(_path: P,
                     _compression_type: CompressionType)
                     -> error::Result<Writer>
        where P: AsRef<path::Path>
    {
        unimplemented!()
    }

    pub fn append<P>(_path: P) -> error::Result<Writer>
        where P: AsRef<path::Path>
    {
        unimplemented!()
    }

    pub fn put(&mut self, _key: &[u8], _value: &[u8]) -> error::Result<()> {
        unimplemented!()
    }

    pub fn delete(&mut self, _key: &[u8]) -> error::Result<()> {
        unimplemented!()
    }

    pub fn flush(&mut self) -> error::Result<()> {
        unimplemented!()
    }
}

impl Reader {
    pub fn open<P>(path: P) -> error::Result<Reader>
        where P: AsRef<path::Path>
    {
        let path = path.as_ref();
        let header = Header::load(path)?;

        let file = fs::File::open(path)?;
        let prot = memmap::Protection::Read;
        let len = header.data_end as usize;
        let map = memmap::Mmap::open_with_offset(&file, prot, 0, len)?;

        debug!("Opened log {:?} with header {:?} map {:?}",
               path,
               header,
               map);

        Ok(Reader {
            header: header,
            map: map,
        })
    }

    pub fn max_key_len(&self) -> u64 {
        self.header.max_key_len
    }

    pub fn max_value_len(&self) -> u64 {
        self.header.max_value_len
    }

    pub fn compression_block_size(&self) -> u32 {
        match self.header.compression_type {
            CompressionType::Snappy(size) => size,
            _ => 0,
        }
    }

    pub fn compression_type(&self) -> CompressionType {
        self.header.compression_type
    }

    pub fn num_puts(&self) -> u64 {
        self.header.num_puts
    }

    pub fn num_deletes(&self) -> u64 {
        self.header.num_deletes
    }

    pub fn num_entries(&self) -> u64 {
        self.num_puts() + self.num_deletes()
    }

    pub fn entries(&self) -> Entries {
        match self.header.compression_type {
            CompressionType::None => {
                Entries(Box::new(uncompressed::EntryReader::new(self)))
            }
            CompressionType::Snappy(_) => {
                Entries(Box::new(snappy::EntryReader::new(self)))
            }
        }
    }

    pub fn owned_entries(&self) -> OwnedEntries {
        OwnedEntries::new(self.entries())
    }

    pub unsafe fn entries_at(&self, position: usize) -> Entries {
        match self.header.compression_type {
            CompressionType::None => {
                Entries(Box::new(uncompressed::EntryReader::new_at(self,
                                                                   position)))
            }
            CompressionType::Snappy(_) => {
                Entries(Box::new(snappy::EntryReader::new_at(self, position)))
            }
        }
    }

    pub unsafe fn owned_entries_at(&self, position: usize) -> OwnedEntries {
        OwnedEntries::new(self.entries_at(position))
    }
}

impl<A> Entry<A> {
    pub fn key(&self) -> &A {
        match *self {
            Entry::Put(ref k, _) => k,
            Entry::Delete(ref k) => k,
        }
    }

    pub fn value(&self) -> Option<&A> {
        match *self {
            Entry::Put(_, ref v) => Some(v),
            Entry::Delete(_) => None,
        }
    }
}

impl<'a, A: ?Sized> Entry<borrow::Cow<'a, A>>
    where A: ToOwned
{
    pub fn into_owned(self) -> Entry<A::Owned> {
        match self {
            Entry::Put(k, v) => Entry::Put(k.into_owned(), v.into_owned()),
            Entry::Delete(k) => Entry::Delete(k.into_owned()),
        }
    }
}

impl<'a> Entries<'a> {
    pub fn try_next(&mut self)
                    -> error::Result<Option<Entry<borrow::Cow<'a, [u8]>>>> {
        self.0.next()
    }

    pub fn skip_next(&mut self) -> error::Result<()> {
        self.0.skip_next()
    }
}

impl<'a> Iterator for Entries<'a> {
    type Item = error::Result<Entry<borrow::Cow<'a, [u8]>>>;

    fn next(&mut self) -> Option<Self::Item> {
        util::flip_option(self.try_next())
    }
}

impl<'a> OwnedEntries<'a> {
    fn new(entries: Entries<'a>) -> OwnedEntries<'a> {
        OwnedEntries(entries)
    }

    pub fn try_next(&mut self) -> error::Result<Option<Entry<Vec<u8>>>> {
        Ok(self.0.try_next()?.map(|e| e.into_owned()))
    }
}

impl<'a> Iterator for OwnedEntries<'a> {
    type Item = error::Result<Entry<Vec<u8>>>;

    fn next(&mut self) -> Option<Self::Item> {
        util::flip_option(self.try_next())
    }
}
