
const DICTIONARY_SIZE: usize = 128;

struct Dictionary {
    entries: Vec<String>,
}

struct CompressedString {
    dict: std::sync::Arc<Dictionary>,
    text: Vec<u8>,
}
