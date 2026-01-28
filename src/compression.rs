
const DICTIONARY_SIZE: usize = 128;
const UNICODE_ESCAPE_BYTE: u8 = 0x00;

use std::{
    collections::HashMap,
    sync::Arc,
};

struct Dictionary {
    entries: Vec<String>,
}

/// To increase compression, some conventions will be used.
///     1 - After *every word, comes a whitespace.
///     2 - \n, \r, \t are always killed by the split_whitespace() function. 
///     3 - After a comma, closing brackets and quotes, quotation and exclamation marks
///         There's ALWAYS a whitespace.
///
///     *Every word that does not have any punctuation after it. In that case it will be respected.
///
/// The compression is potencially lossy if the input string is not formatted accordingly.
struct CompressedString {
    dict: std::sync::Arc<Dictionary>,
    text: Vec<u8>,
}

impl CompressedString {

    fn compress(string: &str, dict: Arc<Dictionary>) -> CompressedString {

        let mut str_to_index = HashMap::with_capacity(DICTIONARY_SIZE);
        let mut text = Vec::with_capacity(string.len());

        for entry in dict.entries.iter().enumerate() {
            str_to_index.insert(entry.1.clone(), entry.0 + 128);
        }

        for word in string.split_whitespace() {

            // Test the last character
            let to_append = word
                .chars()
                .last()
                .filter(|&last_letter| matches!(last_letter, '\'' | '"' | '!' | '?' | ')' | '-' | ']' | '}' | ':' | ';' | ',' | '.'));

            if let Some(punctuation) = to_append { // === If it has punctuation

                // Actually removes the punctuation
                let clean_word = {
                    let mut word = word.chars();
                    word.next_back();

                    word.collect::<String>()
                };

                if let Some(shorten_word) = str_to_index.get(&clean_word) { // === If it is
                    // in the dictionary
                    text.push(*shorten_word as u8);

                } else {
                    for character in clean_word.chars() {

                        if character.is_ascii() {
                            text.push(character as u8);
                        } else {
                            let mut buffer = [0; 4];
                            let extension = character
                                .encode_utf8(&mut buffer)
                                .as_bytes();
                            text.push(UNICODE_ESCAPE_BYTE);
                            text.extend_from_slice(extension);
                        }
                    }
                }

                text.push(punctuation as u8);

            } else if let Some(shorten_word) = str_to_index.get(word) { // If it is in the
                    // dictionary
                 text.push(*shorten_word as u8);

            } else {

                    for character in word.chars() {
                        if character.is_ascii() {
                            text.push(character as u8);
                        } else {
                            let mut buffer = [0; 4];
                            let extension = character
                                .encode_utf8(&mut buffer)
                                .as_bytes();

                            text.push(UNICODE_ESCAPE_BYTE);
                            text.extend_from_slice(extension);
                        }

                    }

                    text.push(b' ');

            }


        }

        CompressedString {
            dict,
            text,
        }
    }

    fn decompress(&self) -> String {

        let str_len = self.text.len() * 2; // Chances are the string will be at least 2x larger than
        // the compressed version
        let mut return_string = String::with_capacity(str_len);
        let mut text_iter = self.text.iter();

        while let Some(byte) = text_iter.next() {

            match *byte {

                UNICODE_ESCAPE_BYTE => {
                    let mut buffer = [0u8; 4];
                    let first = *text_iter.next().expect("Bad unicode escape.");

                    let len = utf8_len(first);
                    buffer[0] = first;

                    for idx in 1..len {
                        buffer[idx as usize] = *text_iter.next().expect("Bad unicode escape.");
                    }

                    let unicode_char = str::from_utf8(&buffer)
                        .unwrap_or("")
                        .chars()
                        .next()
                        .unwrap();

                    return_string.push(unicode_char);
                }

                b'\'' | b'"' | b'!' | b'?' | b')' | b'-' | b']' | b'}' | b':' | b';' | b',' | b'.' => {

                    return_string.pop();

                    return_string.push(*byte as char);

                }

                1..=0x7F => {

                    return_string.push(*byte as char);

                },

                0x80..=0xFF => {

                    return_string.push_str(&self.dict.entries[(*byte - 0x80) as usize]);

                },

            }

            return_string.push(' ');

        }

        if let Some(maybe_whitespace) = return_string.chars().next_back() && maybe_whitespace == ' ' {
            return_string.pop();
        }

        return_string

    }

}

#[inline]
fn utf8_len(byte: u8) -> u8 {

    byte.leading_ones() as u8

}
