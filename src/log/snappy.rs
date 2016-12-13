use std::borrow;
use std::cmp;
use std::fmt;
use std::io;
use snap;

use log;
use error;
use util;

#[derive(Debug)]
struct Block {
    position: usize,
    data: Vec<u8>,
}

#[derive(Debug)]
struct Blocks<'a> {
    reader: &'a log::Reader,
    next_position: usize,
}

#[derive(Debug)]
struct BlockChunks<'a> {
    blocks: Blocks<'a>,
    current_block: Option<Block>,
    block_offset: usize,
}

pub trait Skip {
    fn skip(&mut self, amount: usize) -> io::Result<()>;
}

#[derive(Debug)]
pub struct EntryReader<R> {
    read: R,
}

impl<'a> Blocks<'a> {
    fn new(reader: &log::Reader) -> Blocks {
        Blocks {
            reader: reader,
            next_position: reader.header.header_size as usize,
        }
    }

    fn new_at(reader: &log::Reader, next_position: usize) -> Blocks {
        Blocks {
            reader: reader,
            next_position: next_position,
        }
    }

    fn try_next(&mut self) -> io::Result<Option<Block>> {
        let position = self.next_position;
        if position >= self.reader.header.data_end as usize {
            Ok(None)
        } else {
            let data = unsafe { self.reader.map.as_slice() };
            trace!("Loading new block at {}", position);

            let (block, size) = read_block(&data[position..])?;

            self.next_position += size;

            Ok(Some(Block {
                position: position,
                data: block,
            }))
        }
    }
}

impl<'a> Iterator for Blocks<'a> {
    type Item = io::Result<Block>;

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

impl<'a> io::Read for BlockChunks<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
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
                    return Ok(0);
                }
            }
        };
        let result_size = cmp::min(buf.len(), slice.len());
        self.block_offset += result_size;
        buf[..result_size].copy_from_slice(&slice[..result_size]);
        Ok(result_size)
    }
}

impl<'a> Skip for BlockChunks<'a> {
    fn skip(&mut self, mut size: usize) -> io::Result<()> {
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
                        bail!(io::Error::new(io::ErrorKind::UnexpectedEof,
                                             "Too few blocks"))
                    }
                }
            };
            let result_size = cmp::min(size, slice_len);
            size -= result_size;
            self.block_offset += result_size;
        }
        Ok(())
    }
}

impl<'a> EntryReader<BlockChunks<'a>> {
    pub fn new(log_reader: &log::Reader) -> EntryReader<BlockChunks> {
        EntryReader { read: BlockChunks::new(Blocks::new(log_reader)) }
    }

    pub fn new_at(log_reader: &log::Reader,
                  position: usize)
                  -> EntryReader<BlockChunks> {
        EntryReader {
            read: BlockChunks::new(Blocks::new_at(log_reader, position)),
        }
    }
}

impl<'a, R> log::EntryReader<'a> for EntryReader<R>
    where R: io::Read + fmt::Debug + Skip
{
    fn next(&mut self)
            -> error::Result<Option<log::Entry<borrow::Cow<'a, [u8]>>>> {
        let a = util::read_vlq(&mut self.read)? as usize;
        let b = util::read_vlq(&mut self.read)? as usize;

        let entry = if a == 0 {
            let mut key = vec![0; b];

            self.read.read_exact(&mut key)?;

            log::Entry::Delete(borrow::Cow::from(key))
        } else {
            let mut key = vec![0; a - 1];
            let mut value = vec![0; b];

            self.read.read_exact(&mut key)?;
            self.read.read_exact(&mut value)?;

            log::Entry::Put(borrow::Cow::from(key), borrow::Cow::from(value))
        };

        Ok(Some(entry))
    }

    fn skip_next(&mut self) -> error::Result<()> {
        let a = util::read_vlq(&mut self.read)? as usize;
        let b = util::read_vlq(&mut self.read)? as usize;

        if a == 0 {
            self.read.skip(b)?;
        } else {
            self.read.skip(a + b - 1)?;
        }

        Ok(())
    }
}

fn read_block(data: &[u8]) -> io::Result<(Vec<u8>, usize)> {
    let mut cursor = io::Cursor::new(data);
    let compressed_size = util::read_vlq(&mut cursor)? as usize;
    let start = cursor.position() as usize;

    let mut decoder = snap::Decoder::new();
    let decompressed =
        decoder.decompress_vec(&data[start..start + compressed_size])?;

    Ok((decompressed, start + compressed_size))
}
