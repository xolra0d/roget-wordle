use crate::{Correctness, DICTIONARY, Guess, Guesser, Word};
use once_cell::sync::OnceCell;
use std::{borrow::Cow, collections::HashMap, ops::Neg};

static INITITIAL: OnceCell<Vec<(Word, usize)>> = OnceCell::new();
static MATCH: OnceCell<HashMap<(Word, Word, [Correctness; 5]), bool>> = OnceCell::new();

pub struct PreCalc {
    remaining: Cow<'static, Vec<(Word, usize)>>,
}

impl PreCalc {
    pub fn new() -> Self {
        Self {
            remaining: Cow::Borrowed(INITITIAL.get_or_init(|| {
                let mut words = Vec::from_iter(DICTIONARY.lines().map(|line| {
                    let (word, count) = line
                        .split_once(' ')
                        .expect("Every line is word + space + frequency ");
                    let count: usize = count.parse().expect("Every count is a number");
                    let word: &[u8; 5] = word
                        .as_bytes()
                        .try_into()
                        .expect("every word should be 5 characters");

                    (*word, count)
                }));
                words.sort_unstable_by_key(|&(_, count)| std::cmp::Reverse(count));
                words
            })),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Candidate {
    word: Word,
    goodness: f64,
}

impl Guesser for PreCalc {
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
        for &(word, _) in &*self.remaining {
            let mut sum = 0.0;
            for pattern in Correctness::patterns() {
                let mut in_pattern_total = 0;
                for &(candidate, count) in &*self.remaining {
                    let matches = MATCH.get_or_init(|| {
                        let words = &INITITIAL.get().unwrap()[..1024];
                        let patterns = Correctness::patterns();

                        let mut out = HashMap::with_capacity(
                            (words.len() * words.len() * patterns.count()) / 2,
                        );
                        for &(word1, _) in words {
                            for &(word2, _) in words {
                                if word2 < word1 {
                                    break;
                                }
                                for pattern in Correctness::patterns() {
                                    let g: Guess = Guess {
                                        word: word1,
                                        mask: pattern,
                                    };
                                    out.insert((word1, word2, pattern), g.matches(&candidate));
                                }
                            }
                        }
                        out
                    });

                    let key = if word < candidate {
                        (word, candidate, pattern)
                    } else {
                        (candidate, word, pattern)
                    };

                    if matches.get(&key).copied().unwrap_or_else(|| {
                        let g: Guess = Guess {
                            word,
                            mask: pattern,
                        };
                        g.matches(&candidate)
                    }) {
                        in_pattern_total += count;
                    }
                }

                if in_pattern_total == 0 {
                    continue;
                }
                let p_of_this_pattern = in_pattern_total as f64 / remaining_count as f64;
                sum += p_of_this_pattern * p_of_this_pattern.log2();
            }
            let goodness = sum.neg();

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
