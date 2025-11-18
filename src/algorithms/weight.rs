use crate::{Correctness, DICTIONARY, Guess, Guesser, Word};
use once_cell::sync::OnceCell;
use std::{borrow::Cow, ops::Neg};

static INITITIAL: OnceCell<Vec<(Word, usize)>> = OnceCell::new();

pub struct Weigtht {
    remaining: Cow<'static, Vec<(Word, usize)>>,
}

impl Weigtht {
    pub fn new() -> Self {
        Self {
            remaining: Cow::Borrowed(INITITIAL.get_or_init(|| {
                Vec::from_iter(DICTIONARY.lines().map(|line| {
                    let (word, count) = line
                        .split_once(' ')
                        .expect("Every line is word + space + frequency ");
                    let count: usize = count.parse().expect("Every count is a number");
                    let word: &[u8; 5] = word
                        .as_bytes()
                        .try_into()
                        .expect("every word should be 5 characters");

                    (*word, count)
                }))
            })),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Candidate {
    word: Word,
    goodness: f64,
}

impl Guesser for Weigtht {
    fn guess(&mut self, history: &[Guess]) -> Word {
        if history.is_empty() {
            return *b"tares";
        }
        if let Some(last) = history.last() {
            if matches!(self.remaining, Cow::Owned(_)) {
                self.remaining
                    .to_mut()
                    .retain(|(word, _)| last.matches(word));
            } else {
                self.remaining = Cow::Owned(
                    self.remaining
                        .iter()
                        .filter(|(word, _)| last.matches(word))
                        .copied()
                        .collect(),
                );
            }
        }

        let remaining_count: usize = self.remaining.iter().map(|&(_, c)| c).sum();

        let mut best: Option<Candidate> = None;
        for &(word, count) in &*self.remaining {
            let mut sum = 0.0;
            for pattern in Correctness::patterns() {
                let mut in_pattern_total = 0;
                for (candidate, count) in &*self.remaining {
                    let g = Guess {
                        word,
                        mask: pattern,
                    };
                    if g.matches(candidate) {
                        in_pattern_total += count;
                    }
                }

                if in_pattern_total == 0 {
                    continue;
                }
                let p_of_this_pattern = in_pattern_total as f64 / remaining_count as f64;
                sum += p_of_this_pattern * p_of_this_pattern.log2();
            }
            let p_word = count as f64 / remaining_count as f64;
            let goodness = sum.neg() * p_word;

            if let Some(c) = best {
                if goodness > c.goodness {
                    best = Some(Candidate { word, goodness })
                }
            } else {
                best = Some(Candidate { word, goodness })
            }
        }
        best.unwrap().word
    }
}
