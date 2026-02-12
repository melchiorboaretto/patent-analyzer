
const DICTIONARY_SIZE: usize = 128;
const UNICODE_ESCAPE_BYTE: u8 = 0x01;

const HEADER_FILENAME: &str = "header.dict";
const DICTIONARY_FILENAME: &str = "table.dict";

use std::{
    collections::{
        BinaryHeap, 
        HashMap
    },
    cmp::Reverse,
    sync::Arc,
};

struct Dictionary {
    entries: Vec<String>,
}

impl<'a> Dictionary {

    pub fn from_strings<C: IntoIterator<Item = &'a str>>(data: C) -> Self {

        // Step 1 - Count all the words.
        let mut counting_map: HashMap<&str, u64> = HashMap::new();

        let data = data.into_iter()
            .map(|s| s.split_whitespace());

        for sentence in data {

            for mut word in sentence {

                if let Some(last_char) = word.chars().last() 
                && matches!(last_char, '\'' | '"' | '!' | '?' | ')' | '-' | ']' | '}' | ':' | ';' | ',' | '.') {

                    // REMINDER: IT WORKS BECAUSE I'M USING ASCII CHARACTERS, IF UNICODE CHARS ARE
                    // USED, IT IS NECESSARY TO REWRITE THIS LOGIC
                    word = &word[..word.len() - 1];

                }
                if let Some(accum) = counting_map.get_mut(word) {
                    *accum += 1;

                } else {
                    counting_map.insert(word, 1);
                }
            }
        }

        // Step 2 - Use a min. heap to order the dictionary
        // The "score" I'm using here is just frequency x word_length.
        let mut heap_full = false;
        let mut heap: BinaryHeap<(Reverse<u64>, &str)> = BinaryHeap::with_capacity(DICTIONARY_SIZE);
        let mut counting_iter = counting_map.iter();

        while !heap_full && let Some(count_next) = counting_iter.next() {

            let score = count_next.0.len() as u64 * count_next.1;
            let ref_score = (score, count_next.0);

            heap.push((Reverse(ref_score.0), ref_score.1));
            if heap.len() >= DICTIONARY_SIZE {
                heap_full = true;
            }

        }

        for remaining in counting_iter {

            let score = remaining.0.len() as u64 * remaining.1;
            let ref_score = (score, remaining.0);

            if let Some(least_value) = heap.peek() {
                if least_value.0 > Reverse(ref_score.0) {
                    heap.pop();
                    heap.push((Reverse(ref_score.0), ref_score.1));
                }
            } else {
                unsafe {
                    std::hint::unreachable_unchecked();
                }
            }
        }

        let dict_vec = heap.into_iter()
            .map(|pair| pair.1.to_string())
            .collect();

        Dictionary {
            entries: dict_vec,
        }

    }

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

    pub fn compress(string: &str, dict: Arc<Dictionary>) -> CompressedString {

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

                if matches!(punctuation, ',' | '.' | '-' | ':') {
                    text.push(b' ');
                }

            } else if let Some(shorten_word) = str_to_index.get(word) { // If it is in the
                    // dictionary
                text.push(*shorten_word as u8);
                text.push(b' ');

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

    pub fn decompress(&self) -> String {

        let str_len = self.text.len() * 2; // Chances are the string will be at least 2x larger than
        // the compressed version
        let mut return_string = String::with_capacity(str_len);
        let mut text_iter = self.text.iter();

        while let Some(byte) = text_iter.next() {

            match *byte {

                0x00 => {},

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

                b'\'' | b'"' | b'!' | b'?' | b')' | b']' | b'}' | b';' => {

                    let last_char = return_string.pop();

                    if let Some(last) = last_char  && last != ' ' {
                        return_string.push(last);
                    }

                    return_string.push(*byte as char);
                    return_string.push(' ');

                }

                0x02..=0x7F => {

                    return_string.push(*byte as char);

                },

                0x80..=0xFF => {

                    return_string.push_str(&self.dict.entries[(*byte - 0x80) as usize]);

                },

            }

        }

        if let Some(maybe_whitespace) = return_string.chars().next_back() && maybe_whitespace == ' ' {
            return_string.pop();
        }

        return_string

    }

}

impl std::fmt::Display for CompressedString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.decompress())
    }
}

#[inline]
fn utf8_len(byte: u8) -> u8 {

    byte.leading_ones() as u8

}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn compress_and_decompress() {

        let test_string = "Gaúcha Zero Hora 28/01/2026 - 16:50h Anvisa aprova cultivo de cannabis para fins medicinais.\
            De acordo com o texto, a produção de cannabis só será autorizada para fins medicinais e farmacêuticos, \
            sendo restrita a pessoas jurídicas. Os estabelecimentos só poderão produzir a quantidade necessária para atender a uma demanda \
            de medicamentos autorizada previamente. Ainda conforme a proposta, o teor de THC deverá ser no máximo de 0,3%. As áreas de cultivo \
            serão limitadas, devendo ser georreferenciadas, fotografadas e monitoradas. Segundo a Anvisa, tratam-se de áreas pequenas, que serão \
            acompanhadas de perto pela agência. Para o transporte dos produtos, a Anvisa informou que será \
            firmada uma parceria com a Polícia Rodoviária Federal.";

        let dict_not_optimal_words = ["de", "a", "o", "que", "e", "do", "da", "em", "um", "para", "é",
            "com", "não", "uma", "os", "no", "se", "na", "por", "mais", "as", "dos", "como", "mas", "foi",
            "ao", "ele", "das", "tem", "à", "seu", "sua", "ou", "ser", "quando", "muito", "nos", "já", "está",
            "eu", "também", "só", "pelo", "pela", "até", "isso", "ela", "entre", "depois", "sem", "mesmo", "aos",
            "ter", "seus", "quem", "nas", "me", "esse", "eles", "estão", "você", "tinha", "foram", "essa", "num",
            "nem", "suas", "meu", "às", "minha", "têm", "numa", "pelos", "elas", "havia", "seja", "qual", "era",
            "fazer", "dois", "toda", "outro", "te", "comigo", "fui", "foi", "estou", "agora", "pois", "deve", "do",
            "diz", "está", "toda", "nossa", "pode", "tão", "alguns", "onde", "aqui", "será", "vida", "antes", "ano",
            "casa", "dia", "homem", "moço", "senhor", "palavra", "filho", "noite", "amigo", "bem", "rua", "vida", "hora",
            "coração", "pai", "pessoa", "mulher", "amor", "verdade", "ideia", "mãe", "marido", "espírito", "fim"];

        let dict = Dictionary {
            entries: dict_not_optimal_words.iter().map(|str| str.to_string()).collect(),
        };

        let overkill_dict = Dictionary::from_strings(vec![test_string]);

        let overcompressed = CompressedString::compress(test_string, Arc::new(overkill_dict));

        let compressed = CompressedString::compress(test_string, Arc::new(dict));

        assert_eq!(compressed.decompress(), test_string);
        assert_eq!(compressed.decompress(), overcompressed.decompress());

        assert_eq!(format!("{}", compressed), format!("{}", test_string));

    }
}
