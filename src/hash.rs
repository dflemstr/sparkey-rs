use std::fs;
use std::io;
use std::path;

use byteorder;
use memmap;
use murmur3;

use error;
use log;

const MAGIC: u32 = 0x9a11318f;
const MAJOR_VERSION: u32 = 1;
const MINOR_VERSION: u32 = 1;
const HEADER_SIZE: u32 = 112;

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum Type {
    Murmur3_32,
    Murmur3_64,
}

#[derive(Debug)]
struct Header {
    major_version: u32,
    minor_version: u32,
    file_identifier: u32,
    hash_seed: u32,
    header_size: u32,

    data_end: u64,
    max_key_len: u64,
    max_value_len: u64,

    garbage_size: u64,
    num_entries: u64,
    address_size: u32,
    hash_size: u32,
    hash_capacity: u64,
    max_displacement: u64,
    num_puts: u64,
    entry_block_bits: u32,
    entry_block_bitmask: u32,
    hash_collisions: u64,
    total_displacement: u64,
    hash_algorithm: Type,
}

#[derive(Debug)]
pub struct Writer;

#[derive(Debug)]
pub struct Reader {
    header: Header,
    map: memmap::Mmap,
    log_reader: log::Reader,
}

impl Type {
    fn hash(&self, data: &[u8], seed: u32) -> u64 {
        let mut cursor = io::Cursor::new(data);
        match *self {
            Type::Murmur3_32 => murmur3::murmur3_32(&mut cursor, seed) as u64,
            Type::Murmur3_64 => {
                use byteorder::ReadBytesExt;

                let mut out = [0; 16];
                murmur3::murmur3_x64_128(&mut cursor, seed, &mut out);
                io::Cursor::new(out)
                    .read_u64::<byteorder::LittleEndian>()
                    .unwrap()
            }
        }
    }

    fn read_hash(&self, data: &[u8]) -> u64 {
        use byteorder::ReadBytesExt;

        let mut cursor = io::Cursor::new(data);
        match *self {
            Type::Murmur3_32 => {
                cursor.read_u32::<byteorder::LittleEndian>().unwrap() as u64
            }
            Type::Murmur3_64 => {
                cursor.read_u64::<byteorder::LittleEndian>().unwrap()
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
            bail!(error::ErrorKind::WrongHashMagicNumber);
        }

        let major_version = file.read_u32::<byteorder::LittleEndian>()?;
        if major_version != MAJOR_VERSION {
            bail!(error::ErrorKind::WrongHashMajorVersion);
        }

        let minor_version = file.read_u32::<byteorder::LittleEndian>()?;
        if minor_version > MINOR_VERSION {
            bail!(error::ErrorKind::UnsupportedHashMinorVersion);
        }

        let file_identifier = file.read_u32::<byteorder::LittleEndian>()?;
        let hash_seed = file.read_u32::<byteorder::LittleEndian>()?;
        let data_end = file.read_u64::<byteorder::LittleEndian>()?;
        let max_key_len = file.read_u64::<byteorder::LittleEndian>()?;
        let max_value_len = file.read_u64::<byteorder::LittleEndian>()?;
        let num_puts = file.read_u64::<byteorder::LittleEndian>()?;
        let garbage_size = file.read_u64::<byteorder::LittleEndian>()?;
        let num_entries = file.read_u64::<byteorder::LittleEndian>()?;

        let address_size = file.read_u32::<byteorder::LittleEndian>()?;
        let hash_size = file.read_u32::<byteorder::LittleEndian>()?;
        let hash_capacity = file.read_u64::<byteorder::LittleEndian>()?;
        let max_displacement = file.read_u64::<byteorder::LittleEndian>()?;
        let entry_block_bits = file.read_u32::<byteorder::LittleEndian>()?;
        let entry_block_bitmask = (1 << entry_block_bits) - 1;
        let hash_collisions = file.read_u64::<byteorder::LittleEndian>()?;
        let total_displacement = file.read_u64::<byteorder::LittleEndian>()?;

        let hash_algorithm = match hash_size {
            4 => Type::Murmur3_32,
            8 => Type::Murmur3_64,
            _ => bail!(error::ErrorKind::HashSizeInvalid),
        };

        let header = Header {
            major_version: major_version,
            minor_version: minor_version,
            file_identifier: file_identifier,
            hash_seed: hash_seed,
            header_size: HEADER_SIZE,

            data_end: data_end,
            max_key_len: max_key_len,
            max_value_len: max_value_len,

            garbage_size: garbage_size,
            num_entries: num_entries,
            address_size: address_size,
            hash_size: hash_size,
            hash_capacity: hash_capacity,
            max_displacement: max_displacement,
            num_puts: num_puts,
            entry_block_bits: entry_block_bits,
            entry_block_bitmask: entry_block_bitmask,
            hash_collisions: hash_collisions,
            total_displacement: total_displacement,
            hash_algorithm: hash_algorithm,
        };

        Ok(header)
    }
}

impl Writer {
    pub fn write<P1, P2>(_hash_path: P1,
                         _log_path: P2,
                         _hash_type: Option<Type>)
                         -> error::Result<()>
        where P1: AsRef<path::Path>,
              P2: AsRef<path::Path>
    {
        unimplemented!()
    }
}

impl Reader {
    pub fn open<P1, P2>(hash_path: P1, log_path: P2) -> error::Result<Reader>
        where P1: AsRef<path::Path>,
              P2: AsRef<path::Path>
    {
        let log_reader = log::Reader::open(log_path)?;

        let hash_path = hash_path.as_ref();
        let header = Header::load(hash_path)?;

        let file = fs::File::open(hash_path)?;
        let prot = memmap::Protection::Read;
        let len = header.data_end as usize;
        let map = memmap::Mmap::open_with_offset(&file, prot, 0, len)?;

        debug!("Opened hash {:?} with header {:?} map {:?}",
               hash_path,
               header,
               map);

        Ok(Reader {
            header: header,
            map: map,
            log_reader: log_reader,
        })
    }

    pub fn log_reader(&self) -> &log::Reader {
        &self.log_reader
    }

    // TODO: this is very stop-gap for now
    pub fn get(&self, key: &[u8]) -> error::Result<Option<Vec<u8>>> {
        let data = unsafe { self.map.as_slice() };
        let hash_table = &data[self.header.header_size as usize..self.header.data_end as usize];

        let hash_capacity = self.header.hash_capacity as usize;
        let hash_size = self.header.hash_size as usize;
        let address_size = self.header.address_size as usize;
        let slot_size = address_size + hash_size;

        let block_bits = self.header.entry_block_bits as usize;
        let block_mask = self.header.entry_block_bitmask as usize;

        let hash = self.header.hash_algorithm.hash(key, self.header.hash_seed);
        let wanted_slot = (hash % self.header.hash_capacity) as usize;

        let mut displacement = 0;
        let mut slot = wanted_slot;

        trace!("Looking up {:?} with hash {} and wanted slot {}",
               key,
               hash,
               wanted_slot);

        loop {
            let stored_hash = self.header
                .hash_algorithm
                .read_hash(&hash_table[slot * slot_size..]);
            let encoded_position = read_pos(&hash_table[slot * slot_size +
                                                        hash_size..],
                                            address_size)?;

            trace!("Slot {} has stored hash {} pointing to {}",
                   slot,
                   stored_hash,
                   encoded_position);

            if stored_hash == hash {
                let block_position = encoded_position >> block_bits;
                let entry_index = encoded_position & block_mask;

                trace!("Checking hash match in block {} index {}",
                       block_position,
                       entry_index);

                let mut entries =
                    unsafe { self.log_reader.entries_at(block_position) };

                for _ in 0..entry_index {
                    entries.skip_next()?;
                }

                if let Some(log::Entry::Put(stored_key, stored_value)) =
                    entries.try_next()? {
                    if stored_key == key {
                        trace!("Found value for {:?}", key);
                        return Ok(Some(stored_value.to_vec()));
                    }
                }
            }

            if get_displacement(hash_capacity, slot, stored_hash) < displacement {
                return Ok(None);
            }

            slot += 1;
            displacement += 1;

            if slot >= hash_capacity {
                slot = 0;
            }
        }
    }

    pub fn entries(&self) -> error::Result<log::Entries> {
        unimplemented!()
    }

    pub fn num_entries(&self) -> u64 {
        self.header.num_entries
    }

    pub fn num_collisions(&self) -> u64 {
        self.header.hash_collisions
    }
}

fn read_pos(data: &[u8], address_size: usize) -> error::Result<usize> {
    use byteorder::ReadBytesExt;

    let mut cursor = io::Cursor::new(data);
    match address_size {
        4 => Ok(cursor.read_u32::<byteorder::LittleEndian>()? as usize),
        8 => Ok(cursor.read_u64::<byteorder::LittleEndian>()? as usize),
        _ => bail!(error::ErrorKind::AddressSizeInvalid),
    }
}

fn get_displacement(capacity: usize, slot: usize, hash: u64) -> usize {
    let wanted_slot = hash as usize % capacity;
    (capacity + slot - wanted_slot) % capacity
}
