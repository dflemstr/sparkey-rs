use std::borrow;
use std::cmp;
use std::result;

use error;

pub trait Chunks {
    fn take_chunk(&mut self, max_size: usize) -> error::Result<&[u8]>;

    fn skip_chunk(&mut self, max_size: usize) -> error::Result<()>;

    fn fill_chunks<'a>(&'a mut self,
                       size: usize)
                       -> error::Result<borrow::Cow<'a, [u8]>> {
        let mut result;
        {
            let first_chunk = self.take_chunk(size)?;
            if (*first_chunk).len() == size {
                // Because of
                // https://github.com/rust-lang/rust/issues/30223 and
                // https://github.com/rust-lang/rfcs/issues/811
                unsafe {
                    return Ok(borrow::Cow::from(&*(first_chunk as *const [u8])));
                }
            } else {
                result = Vec::with_capacity(size);
                result.extend_from_slice(&*first_chunk);
            }
        }

        while result.len() < size {
            let next_chunk = self.take_chunk(size - result.len())?;
            result.extend_from_slice(next_chunk);
        }

        Ok(borrow::Cow::from(result))
    }
}

pub struct SliceChunks<'a>(&'a [u8]);

impl<'a, C> Chunks for &'a mut C
    where C: Chunks
{
    fn take_chunk<'b>(&'b mut self,
                      max_size: usize)
                      -> error::Result<&'b [u8]> {
        (*self).take_chunk(max_size)
    }

    fn skip_chunk(&mut self, size: usize) -> error::Result<()> {
        (*self).skip_chunk(size)
    }
}

impl<'a> SliceChunks<'a> {
    pub fn new(slice: &[u8]) -> SliceChunks {
        SliceChunks(slice)
    }
}

impl<'a> Chunks for SliceChunks<'a> {
    fn take_chunk<'b>(&'b mut self,
                      max_size: usize)
                      -> error::Result<&'b [u8]> {
        let split = cmp::min(max_size, self.0.len());
        let (result, new_slice) = self.0.split_at(split);
        self.0 = new_slice;
        Ok(result)
    }

    fn skip_chunk(&mut self, size: usize) -> error::Result<()> {
        self.0 = &self.0[size..];
        Ok(())
    }
}

pub fn flip_option<A, E>(option: result::Result<Option<A>, E>)
                         -> Option<result::Result<A, E>> {
    match option {
        Ok(None) => None,
        Ok(Some(r)) => Some(Ok(r)),
        Err(e) => Some(Err(e)),
    }
}

pub fn read_vlq<C>(mut chunks: C) -> error::Result<(u64, usize)>
    where C: Chunks
{
    let mut result = 0u64;

    const MAX_LEN: usize = 10;

    for i in 0..MAX_LEN {
        let chunk = chunks.take_chunk(1)?;
        let byte = *chunk.first()
            .ok_or_else(|| error::Error::from(error::ErrorKind::VlqUnderrun))?;

        let value = byte & 0b01111111u8;

        result = result | ((value as u64) << (i * 7));

        if byte == value {
            return Ok((result, i + 1));
        }
    }

    bail!(error::ErrorKind::VlqOverflow);
}

#[cfg(test)]
mod test {
    use super::*;

    use error;

    fn read_vlq_slice(slice: &[u8]) -> error::Result<(u64, usize)> {
        read_vlq(SliceChunks::new(slice))
    }

    #[test]
    fn read_vlq_underflow() {
        let err = read_vlq_slice(&[0b10000000]).unwrap_err();
        assert_matches!(err.kind(), &error::ErrorKind::VlqUnderrun);
    }

    #[test]
    fn read_vlq_slice_overflow() {
        let err = read_vlq_slice(&[0b10000000, 0b10000000, 0b10000000,
                                   0b10000000, 0b10000000, 0b10000000,
                                   0b10000000, 0b10000000, 0b10000000,
                                   0b10000000])
            .unwrap_err();
        assert_matches!(err.kind(), &error::ErrorKind::VlqOverflow);
    }

    #[test]
    fn read_vlq_slice_1_min() {
        let (value, len) = read_vlq_slice(&[0b00000000]).unwrap();
        assert_eq!(0, value);
        assert_eq!(1, len);
    }

    #[test]
    fn read_vlq_slice_1_max() {
        let (value, len) = read_vlq_slice(&[0b01111111]).unwrap();
        assert_eq!(127, value);
        assert_eq!(1, len);
    }

    #[test]
    fn read_vlq_slice_2_min() {
        let (value, len) = read_vlq_slice(&[0b10000000, 0b00000001]).unwrap();
        assert_eq!(128, value);
        assert_eq!(2, len);
    }

    #[test]
    fn read_vlq_slice_2_max() {
        let (value, len) = read_vlq_slice(&[0b11111111, 0b01111111]).unwrap();
        assert_eq!(16383, value);
        assert_eq!(2, len);
    }

    #[test]
    fn read_vlq_slice_3_min() {
        let (value, len) =
            read_vlq_slice(&[0b10000000, 0b10000000, 0b00000001]).unwrap();
        assert_eq!(16384, value);
        assert_eq!(3, len);
    }

    #[test]
    fn read_vlq_slice_3_max() {
        let (value, len) =
            read_vlq_slice(&[0b11111111, 0b11111111, 0b01111111]).unwrap();
        assert_eq!(2097151, value);
        assert_eq!(3, len);
    }

    #[test]
    fn read_vlq_slice_4_min() {
        let (value, len) = read_vlq_slice(&[0b10000000, 0b10000000,
                                            0b10000000, 0b00000001])
            .unwrap();
        assert_eq!(2097152, value);
        assert_eq!(4, len);
    }

    #[test]
    fn read_vlq_slice_4_max() {
        let (value, len) = read_vlq_slice(&[0b11111111, 0b11111111,
                                            0b11111111, 0b01111111])
            .unwrap();
        assert_eq!(268435455, value);
        assert_eq!(4, len);
    }
}
