use nanorand::{Rng, WyRand};

const WORDS: &str = include_str!("words.txt");

pub struct WordPicker {
    words: Vec<Vec<u8>>,
    rng: WyRand,
}

impl WordPicker {
    pub fn new() -> WordPicker {
        let words = WORDS.lines().map(|w| w.as_bytes().to_vec()).collect();

        WordPicker {
            words,
            rng: WyRand::new(),
        }
    }

    pub fn pick_random_word(&mut self) -> Vec<u8> {
        let i = self.rng.generate_range(0..self.words.len());
        self.words[i].clone()
    }
}

#[test]
fn words_are_ascii_alphabetic() {
    let picker = WordPicker::new();
    for word in picker.words {
        if word.iter().any(|b| !b.is_ascii_alphabetic()) {
            let word = String::from_utf8_lossy(&word);
            panic!("Word is not ascii alphabetic: {word}");
        }
    }
}

#[test]
fn no_empty_words() {
    let picker = WordPicker::new();
    for (i, word) in picker.words.iter().enumerate() {
        if word.len() == 0 {
            panic!("Empty word at index {i}");
        }
    }
}

#[test]
fn pick_word_works() {
    let mut picker = WordPicker::new();
    let word = picker.pick_random_word();
    assert!(!word.is_empty());
}
