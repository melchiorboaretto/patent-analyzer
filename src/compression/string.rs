pub mod codec;
pub mod dictionary_manager;
mod dictionary;

const DICTIONARY_SIZE: usize = 128;
const UNICODE_ESCAPE_BYTE: u8 = 0x01;

const FOLDERNAME: &str = "dictionaries";
const HEADER_FILENAME: &str = "header.dict";
const DICTIONARY_FILENAME: &str = "table.dict";

const OFFSET_SIZE_IN_BYTES: u64 = (size_of::<u64>() * 2) as u64;

