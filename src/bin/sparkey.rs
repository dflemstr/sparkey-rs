use log::debug;
use log::error;
use log::warn;
use std::env;
use std::path;
use std::process;
use std::str;
use structopt::clap::arg_enum;
use structopt::StructOpt;

/// An utility for reading and writing sparkey files
#[derive(Debug, StructOpt)]
#[structopt(name = "sparkey")]
struct Options {
    /// The path to a sparkey file to operate on (either the .spi or .spl file, or the path without
    /// an extension)
    path: path::PathBuf,
    #[structopt(subcommand)]
    command: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Shows information about the set of sparkey files
    #[structopt(name = "show")]
    Show,
    /// Gets the value for a key
    #[structopt(name = "get")]
    Get {
        #[structopt(flatten)]
        key: KeyOptions,
        #[structopt(flatten)]
        value_format: ValueFormatOptions,
    },
    /// Puts (appends) the value for a key to the log (.spl) file
    #[structopt(name = "put")]
    Put {
        #[structopt(flatten)]
        key: KeyOptions,
        #[structopt(flatten)]
        value: ValueOptions,

        #[structopt(flatten)]
        index_format: IndexFormatOptions,

        #[structopt(flatten)]
        log_format: LogFormatOptions,

        /// Whether to automatically create the .spl file; otherwise, run `sparkey create` manually
        #[structopt(long = "auto-create", short = "l")]
        auto_create: bool,

        /// Whether to automatically update the .spi file; otherwise, run `sparkey index` manually
        #[structopt(long = "auto-index", short = "i")]
        auto_index: bool,
    },
    /// Dumps all keys and values in the index to stdout
    #[structopt(name = "dump")]
    Dump {
        #[structopt(flatten)]
        key_format: KeyFormatOptions,
        #[structopt(flatten)]
        value_format: ValueFormatOptions,
    },
    /// Creates a new (empty) log (.spl) file
    #[structopt(name = "create")]
    Create {
        /// Whether to automatically also create the .spi file; otherwise, run `sparkey index` manually
        #[structopt(long = "index", short = "i")]
        index: bool,

        #[structopt(flatten)]
        index_format: IndexFormatOptions,

        #[structopt(flatten)]
        log_format: LogFormatOptions,
    },
    /// Indexes entries into the index (.spi) file, making sure it is up to date with the log (.spl)
    /// file
    #[structopt(name = "index")]
    Index {
        #[structopt(flatten)]
        index_format: IndexFormatOptions,
    },
    /// Prunes an existing log (.spl) file, making sure there is only one entry per index (.spi)
    /// file entry, and writes the corresponding index (.spi) file
    #[structopt(name = "prune")]
    Prune {
        #[structopt(flatten)]
        index_format: IndexFormatOptions,

        #[structopt(flatten)]
        log_format: LogFormatOptions,

        /// The path at which the new pruned sparkey file will be written (either the .spi or .spl
        /// file, or the path without an extension)
        output: path::PathBuf,
    },
}

#[derive(Debug, StructOpt)]
struct KeyOptions {
    /// The key to use
    #[structopt(name = "key_data")]
    data: String,

    #[structopt(flatten)]
    format: KeyFormatOptions,
}

#[derive(Debug, StructOpt)]
struct KeyFormatOptions {
    /// Whether the key is formatted as hex, same as `--key-format=hex`
    #[structopt(name = "key_hex", long = "key-hex")]
    hex: bool,

    /// Whether the key is formatted as base64, same as `--key-format=base64`
    #[structopt(name = "key_base64", long = "key-base64")]
    base64: bool,

    /// The encoding of the key
    #[structopt(
        name = "key_format",
        long = "key-format",
        short = "f",
        default_value = "utf8",
        raw(possible_values = "&Format::variants()", case_insensitive = "true")
    )]
    format: Format,
}

#[derive(Debug, StructOpt)]
struct ValueOptions {
    /// The value to use
    #[structopt(name = "value_data")]
    data: String,

    #[structopt(flatten)]
    format: ValueFormatOptions,
}

#[derive(Debug, StructOpt)]
struct ValueFormatOptions {
    /// Whether the value is formatted as hex, same as `--value-format=hex`
    #[structopt(name = "value_hex", long = "value-hex")]
    hex: bool,

    /// Whether the value is formatted as base64, same as `--value-format=base64`
    #[structopt(name = "value_base64", long = "value-base64")]
    base64: bool,

    /// The encoding of the value
    #[structopt(
        name = "value_format",
        long = "value-format",
        short = "F",
        default_value = "utf8",
        raw(possible_values = "&Format::variants()", case_insensitive = "true")
    )]
    format: Format,
}

#[derive(Debug, StructOpt)]
struct IndexFormatOptions {
    /// Hash algorithm to use
    #[structopt(
        long = "hash-algorithm",
        short = "a",
        raw(
            possible_values = "&HashAlgorithm::variants()",
            case_insensitive = "true"
        )
    )]
    hash_algorithm: Option<HashAlgorithm>,
}

#[derive(Debug, StructOpt)]
struct LogFormatOptions {
    /// Compression algorithm to use when creating the log file
    #[structopt(
        long = "compression-algorithm",
        short = "c",
        default_value = "none",
        raw(
            possible_values = "&CompressionAlgorithm::variants()",
            case_insensitive = "true"
        )
    )]
    compression_algorithm: CompressionAlgorithm,

    /// Compression block size to use
    #[structopt(long = "compression-block-size", short = "b", default_value = "4096")]
    compression_block_size: u32,
}

arg_enum! {
    #[derive(Clone, Copy, Debug)]
    #[allow(non_camel_case_types)]
    enum Format {
        utf8,
        hex,
        base64,
    }
}

arg_enum! {
    #[derive(Clone, Copy, Debug)]
    #[allow(non_camel_case_types)]
    enum CompressionAlgorithm {
        none,
        snappy,
    }
}

arg_enum! {
    #[derive(Clone, Copy, Debug)]
    #[allow(non_camel_case_types)]
    enum HashAlgorithm {
        murmur3_32,
        murmur3_64,
    }
}

fn main() {
    match run() {
        Ok(()) => (),
        Err(e) => {
            for cause in e.iter_chain() {
                error!("{}", cause);
            }
            process::exit(1)
        }
    }
}

fn run() -> Result<(), failure::Error> {
    init_log();
    let options: Options = Options::from_args();

    let path = options.path;

    let index_file = path.with_extension("spi");
    let log_file = path.with_extension("spl");

    debug!(
        "using index file {:?} and log file {:?}",
        index_file, log_file
    );

    match options.command {
        Command::Show => {
            if index_file.exists() {
                let reader = sparkey::hash::Reader::open(index_file, log_file)?;
                show_log(reader.log_reader());
                show_index(&reader);
            } else {
                warn!("index file not found; will only show log file");
                let reader = sparkey::log::Reader::open(log_file)?;
                show_log(&reader);
            }
        }
        Command::Get { key, value_format } => {
            let reader = sparkey::hash::Reader::open(index_file, log_file)?;
            let key = decode(key.format.to_format(), key.data)?;
            debug!("raw key is {:?}", key);
            let value = reader.get(&key)?;

            if let Some(value) = value {
                debug!("raw value is {:?}", value);
                println!("{}", encode(value_format.to_format(), value)?);
            } else {
                return Err(failure::err_msg(format!("key not found: {:?}", key)));
            }
        }
        Command::Put {
            key,
            value,
            index_format,
            log_format,
            auto_create,
            auto_index,
        } => {
            let mut writer = if !log_file.exists() && auto_create {
                sparkey::log::Writer::create(
                    &log_file,
                    log_format.compression_algorithm.into(),
                    log_format.compression_block_size,
                )?
            } else {
                sparkey::log::Writer::append(&log_file)?
            };
            let key = decode(key.format.to_format(), key.data)?;
            debug!("raw key is {:?}", key);
            let value = decode(value.format.to_format(), value.data)?;
            debug!("raw value is {:?}", value);

            writer.put(&key, &value)?;
            writer.flush()?;
            drop(writer);

            if auto_index {
                debug!("performing automatic index");
                sparkey::hash::Writer::write(
                    &index_file,
                    &log_file,
                    index_format.hash_algorithm.map(From::from),
                )?;
            }
        }
        Command::Dump {
            key_format,
            value_format,
        } => {
            let reader = sparkey::hash::Reader::open(index_file, log_file)?;
            let key_format = key_format.to_format();
            let value_format = value_format.to_format();

            for entry in reader.entries()? {
                let entry = entry?;
                println!(
                    "{}\t{}",
                    encode(key_format, entry.key)?,
                    encode(value_format, entry.value)?
                );
            }
        }
        Command::Create {
            index,
            index_format,
            log_format,
        } => {
            sparkey::log::Writer::create(
                &log_file,
                log_format.compression_algorithm.into(),
                log_format.compression_block_size,
            )?
            .flush()?;

            if index {
                sparkey::hash::Writer::write(
                    &index_file,
                    &log_file,
                    index_format.hash_algorithm.map(From::from),
                )?;
            }
        }
        Command::Index { index_format } => {
            sparkey::hash::Writer::write(
                &index_file,
                &log_file,
                index_format.hash_algorithm.map(From::from),
            )?;
        }
        Command::Prune {
            index_format,
            log_format,
            output,
        } => {
            let output_index = output.with_extension("spi");
            let output_log = output.with_extension("spl");

            let reader = sparkey::hash::Reader::open(index_file, log_file)?;
            let mut writer = sparkey::log::Writer::create(
                &output_log,
                log_format.compression_algorithm.into(),
                log_format.compression_block_size,
            )?;

            for entry in reader.entries()? {
                let entry = entry?;
                writer.put(&entry.key, &entry.value)?;
            }

            debug!("writing index");

            sparkey::hash::Writer::write(
                &output_index,
                &output_log,
                index_format.hash_algorithm.map(From::from),
            )?;
        }
    }

    Ok(())
}

fn init_log() {
    let mut builder = pretty_env_logger::formatted_builder();
    builder.filter_module(module_path!(), log::LevelFilter::Info);

    if let Ok(var) = env::var("RUST_LOG") {
        builder.parse_filters(&var);
    }

    if let Ok(var) = env::var("RUST_LOG_STYLE") {
        builder.parse_write_style(&var);
    }

    if let Ok(var) = env::var("SPARKEY_LOG") {
        builder.parse_filters(&var);
    }

    if let Ok(var) = env::var("SPARKEY_LOG_STYLE") {
        builder.parse_write_style(&var);
    }

    builder.init()
}

fn show_index(reader: &sparkey::hash::Reader) {
    println!("index_num_entries\t{}", reader.num_entries());
    println!("index_num_collisions\t{}", reader.num_collisions());
}

fn show_log(reader: &sparkey::log::Reader) {
    println!("log_max_key_len\t{}", reader.max_key_len());
    println!("log_max_value_len\t{}", reader.max_value_len());
    println!(
        "log_compression_block_size\t{}",
        reader.compression_block_size()
    );
    println!("log_compression_type\t{}", reader.compression_type());
}

fn decode(format: Format, data: String) -> Result<bytes::BytesMut, failure::Error> {
    match format {
        Format::utf8 => Ok(data.into_bytes().into()),
        Format::hex => Ok(hex::decode(data)?.into()),
        Format::base64 => Ok(base64::decode(&data)?.into()),
    }
}

fn encode(format: Format, data: bytes::BytesMut) -> Result<String, failure::Error> {
    match format {
        Format::utf8 => Ok(String::from_utf8(data.to_vec())?),
        Format::hex => Ok(hex::encode(data)),
        Format::base64 => Ok(base64::encode(&data)),
    }
}

impl KeyFormatOptions {
    fn to_format(&self) -> Format {
        if self.hex {
            Format::hex
        } else if self.base64 {
            Format::base64
        } else {
            self.format
        }
    }
}

impl ValueFormatOptions {
    fn to_format(&self) -> Format {
        if self.hex {
            Format::hex
        } else if self.base64 {
            Format::base64
        } else {
            self.format
        }
    }
}

impl From<CompressionAlgorithm> for sparkey::log::CompressionType {
    fn from(value: CompressionAlgorithm) -> Self {
        match value {
            CompressionAlgorithm::none => sparkey::log::CompressionType::None,
            CompressionAlgorithm::snappy => sparkey::log::CompressionType::Snappy,
        }
    }
}

impl From<HashAlgorithm> for sparkey::hash::Type {
    fn from(value: HashAlgorithm) -> Self {
        match value {
            HashAlgorithm::murmur3_32 => sparkey::hash::Type::Murmur3_32,
            HashAlgorithm::murmur3_64 => sparkey::hash::Type::Murmur3_64,
        }
    }
}
