
use crate::compression::string::{

    DICTIONARY_SIZE,

    dictionary::{
        Dictionary,
    }

};

use std::{

    cmp::Reverse,

    collections::{
        HashMap,
        BinaryHeap,
    },

    sync::OnceLock,

};

// Implements In Memory functions
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

                    // REMINDER: IT WORKS BECAUSE I'M USING ASCII CHARACTERS INSIDE THE MATCH,
                    // IF UNICODE CHARS ARE USED, IT IS NECESSARY TO REWRITE THIS LOGIC
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
            lookup_map: OnceLock::new(),
        }

    }

}

