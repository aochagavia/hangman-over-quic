use std::collections::HashSet;

pub struct GameState {
    word: Vec<u8>,
    correct_guesses: HashSet<u8>,
    lives_left: u8,
}

impl GameState {
    pub fn new(word: Vec<u8>) -> GameState {
        GameState {
            word,
            correct_guesses: HashSet::new(),
            lives_left: 5,
        }
    }

    pub fn word_len(&self) -> u8 {
        self.word.len() as u8
    }

    pub fn handle_guess(&mut self, guess: u8) -> Vec<u8> {
        let guess = guess.to_ascii_lowercase();

        let mut indexes = Vec::new();
        for (i, &ch) in self.word.iter().enumerate() {
            if guess == ch {
                indexes.push(i as u8);
            }
        }

        if indexes.is_empty() {
            self.lives_left = self.lives_left.wrapping_sub(1);
        } else {
            self.correct_guesses.insert(guess);
        }

        indexes
    }

    pub fn is_running(&self) -> bool {
        !self.player_won() && !self.player_lost()
    }

    pub fn player_won(&self) -> bool {
        self.word.iter().all(|ch| self.correct_guesses.contains(ch))
    }

    pub fn player_lost(&self) -> bool {
        self.lives_left == u8::MAX
    }
}

#[test]
fn game_win_flow() {
    let mut state = GameState::new(b"alpaca".to_vec());

    // Correct guess
    let indexes = state.handle_guess(b'A');
    assert_eq!(indexes.len(), 3);
    assert_eq!(indexes, &[0, 3, 5]);

    // One wrong guess
    let indexes = state.handle_guess(b's');
    assert!(indexes.is_empty());

    // Correct guess
    let indexes = state.handle_guess(b'p');
    assert_eq!(indexes.len(), 1);
    assert_eq!(indexes, &[2]);

    // Correct guess
    let indexes = state.handle_guess(b'c');
    assert_eq!(indexes.len(), 1);
    assert_eq!(indexes, &[4]);

    // Sanity check
    assert!(!state.player_won());
    assert!(state.is_running());

    // Correct guess
    let indexes = state.handle_guess(b'l');
    assert_eq!(indexes.len(), 1);
    assert_eq!(indexes, &[1]);

    assert!(state.player_won());
    assert!(!state.is_running());
}

#[test]
fn game_lose_flow() {
    let mut state = GameState::new(b"alpaca".to_vec());

    // Correct guess
    state.handle_guess(b'a');

    // 5 wrong guesses
    for _ in 0..5 {
        state.handle_guess(b's');
    }

    // Sanity check
    assert!(!state.player_lost());
    assert!(state.is_running());

    // 6th wrong guess
    state.handle_guess(b's');

    assert!(state.player_lost());
    assert!(!state.is_running());
}
