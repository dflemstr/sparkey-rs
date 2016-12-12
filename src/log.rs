use std::borrow;
use std::cmp;
use std::fs;
use std::io;
use std::path;

use byteorder;
use memmap;
use snap;

use error;
use hash;
use util;

const MAGIC: u32 = 0x49b39c95;
const MAJOR_VERSION: u32 = 1;
const MINOR_VERSION: u32 = 0;
const HEADER_SIZE: u32 = 84;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CompressionType {
    None,
    Snappy,
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
    compression_block_size: u32,
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
struct Block<'a> {
    position: usize,
    data: borrow::Cow<'a, [u8]>,
}

#[derive(Debug)]
struct Blocks<'a> {
    reader: &'a Reader,
    next_position: usize,
}

#[derive(Debug)]
struct BlockChunks<'a> {
    blocks: Blocks<'a>,
    current_block: Option<Block<'a>>,
    block_offset: usize,
}

#[derive(Debug)]
pub enum Entry<A> {
    Put(A, A),
    Delete(A),
}

#[derive(Debug)]
pub struct Entries<'a> {
    chunks: BlockChunks<'a>,
    hash_reader: Option<&'a hash::Reader>,
}

#[derive(Debug)]
pub struct OwnedEntries<'a>(Entries<'a>);

impl CompressionType {
    fn read_block<'a>(&self,
                      data: &'a [u8])
                      -> error::Result<(borrow::Cow<'a, [u8]>, usize)> {
        match *self {
            CompressionType::None => {
                let block = borrow::Cow::from(data);
                let size = block.len();
                trace!("Read uncompressed block of size {}, stored size {}",
                       block.len(),
                       size);
                Ok((block, size))
            }
            CompressionType::Snappy => {
                let (compressed_size, length) =
                    util::read_vlq(util::SliceChunks::new(data))?;
                let data = &data[length..compressed_size as usize + length];
                let mut decoder = snap::Decoder::new();

                let block = borrow::Cow::from(decoder.decompress_vec(data)?);
                let size = length + compressed_size as usize;
                trace!("Read Snappy compressed block of size {}, stored size \
                        {}",
                       (&*block as &[u8]).len(),
                       size);
                Ok((block, size))
            }
        }
    }
}

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
            1 => CompressionType::Snappy,
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
            compression_block_size: compression_block_size,
            put_size: put_size,
            header_size: HEADER_SIZE,
            max_entries_per_block: max_entries_per_block,
        };

        Ok(header)
    }
}

impl Writer {
    pub fn create<P>(_path: P,
                     _compression_type: CompressionType,
                     _compression_block_size: u32)
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
        self.header.compression_block_size
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

    fn blocks(&self) -> Blocks {
        Blocks::new(self)
    }

    fn block_chunks(&self) -> BlockChunks {
        BlockChunks::new(self.blocks())
    }

    pub fn entries(&self) -> Entries {
        Entries::new(self.block_chunks())
    }

    pub fn owned_entries(&self) -> OwnedEntries {
        OwnedEntries::new(self.entries())
    }

    pub unsafe fn entries_at(&self, position: usize) -> Entries {
        Entries::new(BlockChunks::new(Blocks::new_at(self, position)))
    }

    pub unsafe fn owned_entries_at(&self, position: usize) -> OwnedEntries {
        OwnedEntries::new(self.entries_at(position))
    }
}

impl<'a> Blocks<'a> {
    fn new(reader: &Reader) -> Blocks {
        Blocks {
            reader: reader,
            next_position: reader.header.header_size as usize,
        }
    }

    fn new_at(reader: &Reader, next_position: usize) -> Blocks {
        Blocks {
            reader: reader,
            next_position: next_position,
        }
    }

    fn try_next(&mut self) -> error::Result<Option<Block<'a>>> {
        let position = self.next_position;
        if position >= self.reader.header.data_end as usize {
            Ok(None)
        } else {
            let data = unsafe { self.reader.map.as_slice() };
            let compression_type = self.reader
                .header
                .compression_type;
            trace!("Loading new block at {} with compression type {:?}",
                   position,
                   compression_type);

            let (block, size) = compression_type.read_block(&data[position..])?;

            self.next_position += size;

            Ok(Some(Block {
                position: position,
                data: block,
            }))
        }
    }
}

impl<'a> Iterator for Blocks<'a> {
    type Item = error::Result<Block<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        util::flip_option(self.try_next())
    }
}

impl<'a> BlockChunks<'a> {
    fn new(blocks: Blocks<'a>) -> BlockChunks {
        BlockChunks {
            blocks: blocks,
            current_block: None,
            block_offset: 0,
        }
    }
}

impl<'a> util::Chunks for BlockChunks<'a> {
    fn take_chunk<'b>(&'b mut self,
                      max_size: usize)
                      -> error::Result<&'b [u8]> {
        let slice = match self.current_block {
            Some(ref block) if self.block_offset < block.data.len() => {
                &block.data[self.block_offset..]
            }
            _ => {
                self.block_offset = 0;
                self.current_block = self.blocks.try_next()?;
                if let Some(ref block) = self.current_block {
                    &block.data
                } else {
                    return Ok(&[]);
                }
            }
        };
        let result_size = cmp::min(max_size, slice.len());
        self.block_offset += result_size;
        Ok(&slice[..result_size])
    }

    fn skip_chunk(&mut self, mut size: usize) -> error::Result<()> {
        while size > 0 {
            let slice_len = match self.current_block {
                Some(ref block) if self.block_offset < block.data.len() => {
                    block.data[self.block_offset..].len()
                }
                _ => {
                    self.block_offset = 0;
                    self.current_block = self.blocks.try_next()?;
                    if let Some(ref block) = self.current_block {
                        block.data.len()
                    } else {
                        bail!(error::ErrorKind::LogTooSmall)
                    }
                }
            };
            size -= slice_len;
        }
        Ok(())
    }
}

impl<A> Entry<A> {
    pub fn key(&self) -> &A {
        match *self {
            Entry::Put(ref k, _) => k,
            Entry::Delete(ref k) => k,
        }
    }

    pub fn value(&self) -> &A {
        match *self {
            Entry::Put(_, ref v) => v,
            Entry::Delete(_) => panic!("A delete entry has no value"),
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
    fn new(chunks: BlockChunks<'a>) -> Entries<'a> {
        Entries {
            chunks: chunks,
            hash_reader: None,
        }
    }

    pub fn try_next<'b>
        (&'b mut self)
         -> error::Result<Option<Entry<borrow::Cow<'b, [u8]>>>> {
        use util::Chunks;

        let (a, _) = util::read_vlq(&mut self.chunks)?;
        let a = a as usize;
        let (b, _) = util::read_vlq(&mut self.chunks)?;
        let b = b as usize;

        let entry = if a == 0 {
            let key = self.chunks.fill_chunks(b)?;
            Entry::Delete(key)
        } else {
            let (key, value) = match self.chunks.fill_chunks(a + b - 1)? {
                borrow::Cow::Owned(mut k) => {
                    let v = k.split_off(a - 1);
                    (borrow::Cow::from(k), borrow::Cow::from(v))
                }
                borrow::Cow::Borrowed(entry) => {
                    let (k, v) = entry.split_at(a - 1);
                    (borrow::Cow::from(k), borrow::Cow::from(v))
                }
            };
            Entry::Put(key, value)
        };

        Ok(Some(entry))
    }

    pub fn skip_next(&mut self) -> error::Result<()> {
        use util::Chunks;

        let (a, _) = util::read_vlq(&mut self.chunks)?;
        let a = a as usize;
        let (b, _) = util::read_vlq(&mut self.chunks)?;
        let b = b as usize;

        if a == 0 {
            self.chunks.skip_chunk(b)?;
        } else {
            self.chunks.skip_chunk(a + b - 1)?;
        }

        Ok(())
    }

    pub fn next<'b>(&'b mut self)
                    -> Option<error::Result<Entry<borrow::Cow<'b, [u8]>>>> {
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
