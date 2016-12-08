#![recursion_limit="65536"]

#[macro_use]
extern crate error_chain;
extern crate libc;
extern crate sparkey_sys;

pub mod error;
pub mod log;
pub mod hash;
mod util;

#[cfg(test)]
mod test {
    use super::*;

    extern crate tempdir;

    #[test]
    fn roundtrip() {
        let dir = tempdir::TempDir::new("sparkey-rs").unwrap();
        let log = dir.path().join("data.spl");
        let hash = dir.path().join("data.spi");

        {
            let mut writer =
                log::Writer::create(&log, log::CompressionType::None, 0)
                    .unwrap();
            writer.put(&[1], &[2, 3, 4, 5]).unwrap();
            writer.put(&[6], &[7, 8, 9, 10]).unwrap();
        }
        hash::Writer::write(&hash, &log, None).unwrap();

        let reader = hash::Reader::open(&hash, &log).unwrap();

        assert_eq!(Some(vec![2, 3, 4, 5]), reader.get(&[1]).unwrap());
        assert_eq!(Some(vec![7, 8, 9, 10]), reader.get(&[6]).unwrap());
    }

    #[test]
    fn roundtrip_compressed() {
        let dir = tempdir::TempDir::new("sparkey-rs").unwrap();
        let log = dir.path().join("data.spl");
        let hash = dir.path().join("data.spi");

        {
            let mut writer =
                log::Writer::create(&log, log::CompressionType::Snappy, 1024)
                .unwrap();
            writer.put(&[1], &[2, 3, 4, 5]).unwrap();
            writer.put(&[6], &[7, 8, 9, 10]).unwrap();
        }
        hash::Writer::write(&hash, &log, None).unwrap();

        let reader = hash::Reader::open(&hash, &log).unwrap();

        assert_eq!(Some(vec![2, 3, 4, 5]), reader.get(&[1]).unwrap());
        assert_eq!(Some(vec![7, 8, 9, 10]), reader.get(&[6]).unwrap());
    }
}
