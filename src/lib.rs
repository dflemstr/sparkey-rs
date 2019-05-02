#[macro_use]
extern crate failure;
extern crate libc;
extern crate sparkey_sys;

pub mod error;
pub mod hash;
pub mod log;
mod util;

#[cfg(test)]
mod test {
    use super::*;

    extern crate tempdir;

    use std::fs;
    use std::io;
    use std::path;
    use std::str;

    #[test]
    fn roundtrip() {
        let dir = tempdir::TempDir::new("sparkey-rs").unwrap();
        let log = dir.path().join("data.spl");
        let hash = dir.path().join("data.spi");

        {
            let mut writer = log::Writer::create(&log, log::CompressionType::None, 0).unwrap();
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
            let mut writer = log::Writer::create(&log, log::CompressionType::Snappy, 1024).unwrap();
            writer.put(&[1], &[2, 3, 4, 5]).unwrap();
            writer.put(&[6], &[7, 8, 9, 10]).unwrap();
        }
        hash::Writer::write(&hash, &log, None).unwrap();

        let reader = hash::Reader::open(&hash, &log).unwrap();

        assert_eq!(Some(vec![2, 3, 4, 5]), reader.get(&[1]).unwrap());
        assert_eq!(Some(vec![7, 8, 9, 10]), reader.get(&[6]).unwrap());
    }

    #[test]
    fn read_small() {
        use std::io::BufRead;

        let dir = path::Path::new("testdata");
        let log = dir.join("small.spl");
        let hash = dir.join("small.spi");
        let csv = dir.join("small.csv");
        let csv_file = fs::File::open(csv).unwrap();

        let reader = hash::Reader::open(&hash, &log).unwrap();

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
    fn write_small() {
        use std::io::BufRead;

        let tmp_dir = tempdir::TempDir::new("sparkey-rs").unwrap();
        let actual_log = tmp_dir.path().join("data.spl");
        let actual_hash = tmp_dir.path().join("data.spi");

        let dir = path::Path::new("testdata");
        let expected_log = dir.join("small.spl");
        let expected_hash = dir.join("small.spi");
        let csv = dir.join("small.csv");
        let csv_file = fs::File::open(csv).unwrap();

        {
            let mut writer =
                log::Writer::create(&actual_log, log::CompressionType::Snappy, 1024).unwrap();

            for line in io::BufReader::new(csv_file).lines() {
                let line = line.unwrap();
                let mut parts = line.splitn(2, ",");
                let key = parts.next().unwrap();
                let value = parts.next().unwrap();

                writer.put(key.as_bytes(), value.as_bytes()).unwrap();
            }
        }

        hash::Writer::write(&actual_hash, &actual_log, None).unwrap();

        let expected_reader = hash::Reader::open(&expected_hash, &expected_log).unwrap();
        let actual_reader = hash::Reader::open(&actual_hash, &actual_log).unwrap();

        for expected_entry in expected_reader.entries().unwrap() {
            let expected_entry = expected_entry.unwrap();

            let actual_value = actual_reader.get(&expected_entry.key).unwrap().unwrap();

            assert_eq!(expected_entry.value, actual_value);
        }

        for actual_entry in actual_reader.entries().unwrap() {
            let actual_entry = actual_entry.unwrap();

            let expected_value = expected_reader.get(&actual_entry.key).unwrap().unwrap();

            assert_eq!(expected_value, actual_entry.value);
        }
    }
}
