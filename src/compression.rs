
const DICTIONARY_SIZE: usize = 128;

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
                            text.push(0x00);
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

                            text.push(0x00);
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

}
