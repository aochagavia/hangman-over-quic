use std::collections::HashMap;

pub struct GameState {
    pub word_len: u8,
    pub lives_left: u8,
    pub correct_guesses: HashMap<u8, u8>,
    pub all_guesses: String,
}

impl GameState {
    pub fn with_word_len(word_len: u8) -> GameState {
        GameState {
            lives_left: 5,
            correct_guesses: HashMap::new(),
            all_guesses: String::new(),
            word_len,
        }
    }

    pub fn display_word(&self) -> String {
        let mut buf = String::with_capacity(self.word_len as usize);
        for i in 0..self.word_len {
            if let Some(ch) = self.correct_guesses.get(&i) {
                buf.push(*ch as char);
            } else {
                buf.push('_');
            }
        }

        buf
    }

    // Returns true if the guess was correct
    pub fn handle_guess_result(&mut self, guessed_char: u8, found_indexes: &[u8]) -> bool {
        // Keep track of all guessed characters, so we can display them to the user
        if !self.all_guesses.is_empty() {
            self.all_guesses.push_str(", ");
        }
        self.all_guesses.push(guessed_char as char);

        // Reveal the guessed character at the found indexes
        for &i in found_indexes {
            self.correct_guesses.insert(i, guessed_char);
        }

        let correct_guess = !found_indexes.is_empty();
        if !correct_guess {
            self.lives_left = self.lives_left.wrapping_sub(1);
        }

        correct_guess
    }

    pub fn is_running(&self) -> bool {
        !self.player_won() && !self.player_lost()
    }

    pub fn player_won(&self) -> bool {
        self.correct_guesses.len() == self.word_len as usize
    }

    pub fn player_lost(&self) -> bool {
        self.lives_left == u8::MAX
    }
}

#[test]
fn game_win_flow() {
    let mut state = GameState::with_word_len(6);

    // Correct guess
    let guess_result = state.handle_guess_result(b'a', &[0, 3, 5]);
    assert!(guess_result);
    assert_eq!(state.display_word(), "a__a_a");
    assert_eq!(state.lives_left, 5);

    let guess_result = state.handle_guess_result(b'b', &[]);
    assert!(!guess_result);
    assert_eq!(state.display_word(), "a__a_a");
    assert_eq!(state.lives_left, 4);

    // Correct guess
    let guess_result = state.handle_guess_result(b'p', &[2]);
    assert!(guess_result);
    assert_eq!(state.display_word(), "a_pa_a");

    // Correct guess
    let guess_result = state.handle_guess_result(b'l', &[1]);
    assert!(guess_result);
    assert_eq!(state.display_word(), "alpa_a");

    // Sanity check
    assert!(state.is_running());

    // Correct guess
    let guess_result = state.handle_guess_result(b'c', &[4]);
    assert!(guess_result);
    assert_eq!(state.display_word(), "alpaca");

    assert!(state.player_won());
    assert!(!state.is_running());
}

#[test]
fn game_lose_flow() {
    let mut state = GameState::with_word_len(6);

    // Correct guess
    let guess_result = state.handle_guess_result(b'a', &[0, 3, 5]);
    assert!(guess_result);

    // 5 wrong guesses
    for i in 0..5 {
        let guess_result = state.handle_guess_result(b's', &[]);
        assert!(!guess_result);
        assert_eq!(state.lives_left, 5 - (i + 1));
    }

    // Sanity check
    assert!(!state.player_lost());
    assert!(state.is_running());

    // 6th wrong guess
    let guess_result = state.handle_guess_result(b's', &[]);
    assert!(!guess_result);

    assert!(state.player_lost());
    assert!(!state.is_running());
}
