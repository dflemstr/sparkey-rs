use std::io;
use std::result;

pub fn flip_option<A, E>(option: result::Result<Option<A>, E>)
                         -> Option<result::Result<A, E>> {
    match option {
        Ok(None) => None,
        Ok(Some(r)) => Some(Ok(r)),
        Err(e) => Some(Err(e)),
    }
}

pub fn read_vlq<R>(mut read: R) -> io::Result<u64>
    where R: io::Read
{
    let mut result = 0u64;

    const MAX_LEN: usize = 10;

    for i in 0..MAX_LEN {
        let mut bytes = [0];
        read.read_exact(&mut bytes)?;

        let byte = bytes[0];
        let value = byte & 0b01111111u8;

        result = result | ((value as u64) << (i * 7));

        if byte == value {
            return Ok(result);
        }
    }

    bail!(io::Error::new(io::ErrorKind::InvalidData, "VLQ overflow"));
}

#[cfg(test)]
mod test {
    use super::*;

    use std::io;

    fn read_vlq_slice(slice: &[u8]) -> io::Result<(u64, usize)> {
        let mut cursor = io::Cursor::new(slice);
        let value = read_vlq(&mut cursor)?;
        let length = cursor.position() as usize;
        Ok((value, length))
    }

    #[test]
    fn read_vlq_underrun() {
        let err = read_vlq_slice(&[0b10000000]).unwrap_err();
        assert_matches!(err.kind(), io::ErrorKind::UnexpectedEof);
    }

    #[test]
    fn read_vlq_slice_overflow() {
        let err = read_vlq_slice(&[0b10000000, 0b10000000, 0b10000000,
                                   0b10000000, 0b10000000, 0b10000000,
                                   0b10000000, 0b10000000, 0b10000000,
                                   0b10000000])
            .unwrap_err();
        assert_matches!(err.kind(), io::ErrorKind::InvalidData);
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
