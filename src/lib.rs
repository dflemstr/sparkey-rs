#![recursion_limit="65536"]

extern crate byteorder;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log as logger;
extern crate memmap;
extern crate murmur3;
extern crate snap;

#[cfg(test)]
#[cfg_attr(test, macro_use)]
extern crate assert_matches;

pub mod error;
pub mod log;
pub mod hash;
mod util;

#[cfg(test)]
mod test {
    use super::*;

    extern crate tempdir;
    extern crate env_logger;

    use std::fs;
    use std::io;
    use std::path;
    use std::str;

    #[test]
    fn roundtrip() {
        let _ = env_logger::init();
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

        let mut reader = hash::Reader::open(&hash, &log).unwrap();

        assert_eq!(Some(vec![2u8, 3, 4, 5]), reader.get(&[1]).unwrap());
        assert_eq!(Some(vec![7u8, 8, 9, 10]), reader.get(&[6]).unwrap());
    }

    #[test]
    fn roundtrip_compressed() {
        let _ = env_logger::init();
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

        let mut reader = hash::Reader::open(&hash, &log).unwrap();

        assert_eq!(Some(vec![2u8, 3, 4, 5]), reader.get(&[1]).unwrap());
        assert_eq!(Some(vec![7u8, 8, 9, 10]), reader.get(&[6]).unwrap());
    }

    #[test]
    fn read_small_hash() {
        use std::io::BufRead;

        let _ = env_logger::init();

        let dir = path::Path::new("testdata");
        let log = dir.join("small.spl");
        let hash = dir.join("small.spi");
        let csv = dir.join("small.csv");
        let csv_file = fs::File::open(csv).unwrap();

        let mut reader = hash::Reader::open(&hash, &log).unwrap();

        for line in io::BufReader::new(csv_file).lines() {
            let line = line.unwrap();
            let mut parts = line.splitn(2, ",");
            let key = parts.next().unwrap();
            let expected = parts.next().unwrap();
            let actual_bytes = reader.get(&key.as_bytes()).unwrap().unwrap();
            let actual = str::from_utf8(&actual_bytes).unwrap();

            assert_eq!(expected, actual);
        }
    }

    #[test]
    fn read_small_log() {
        use std::io::BufRead;

        let _ = env_logger::init();

        let dir = path::Path::new("testdata");
        let log = dir.join("small.spl");
        let csv = dir.join("small.csv");
        let csv_file = fs::File::open(csv).unwrap();

        let reader = log::Reader::open(&log).unwrap();
        let mut entries = reader.entries();

        for line in io::BufReader::new(csv_file).lines() {
            let line = line.unwrap();
            let mut parts = line.splitn(2, ",");
            let expected_key = parts.next().unwrap();
            let expected_value = parts.next().unwrap();

            let actual_entry = entries.try_next().unwrap().unwrap();
            let actual_key = str::from_utf8(actual_entry.key()).unwrap();
            let actual_value = str::from_utf8(actual_entry.value()).unwrap();

            assert_eq!(expected_key, actual_key);
            assert_eq!(expected_value, actual_value);
        }
    }

    #[test]
    fn write_small() {
        use std::io::BufRead;

        let _ = env_logger::init();

        let tmp_dir = tempdir::TempDir::new("sparkey-rs").unwrap();
        let actual_log = tmp_dir.path().join("data.spl");
        let actual_hash = tmp_dir.path().join("data.spi");

        let dir = path::Path::new("testdata");
        let expected_log = dir.join("small.spl");
        let expected_hash = dir.join("small.spi");
        let csv = dir.join("small.csv");
        let csv_file = fs::File::open(csv).unwrap();

        {
            let mut writer = log::Writer::create(&actual_log,
                                                 log::CompressionType::Snappy,
                                                 1024)
                .unwrap();

            for line in io::BufReader::new(csv_file).lines() {
                let line = line.unwrap();
                let mut parts = line.splitn(2, ",");
                let key = parts.next().unwrap();
                let value = parts.next().unwrap();

                writer.put(key.as_bytes(), value.as_bytes()).unwrap();
            }
        }

        hash::Writer::write(&actual_hash, &actual_log, None).unwrap();

        let mut expected_reader =
            hash::Reader::open(&expected_hash, &expected_log).unwrap();
        let mut actual_reader = hash::Reader::open(&actual_hash, &actual_log)
            .unwrap();

        {
            let mut expected_entries = expected_reader.entries().unwrap();
            while let Some(expected_entry) =
                expected_entries.try_next().unwrap() {
                let actual_value =
                    actual_reader.get(expected_entry.key()).unwrap().unwrap();

                assert_eq!(expected_entry.value().to_vec(), actual_value);
            }
        }

        let mut actual_entries = actual_reader.entries().unwrap();
        while let Some(actual_entry) = actual_entries.try_next().unwrap() {
            let expected_value =
                expected_reader.get(&actual_entry.key()).unwrap().unwrap();

            assert_eq!(expected_value, actual_entry.value().to_vec());
        }
    }
}
