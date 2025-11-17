use std::usize;

use clap::{Parser, ValueEnum};
use roget::Guesser;

const GAMES: &str = include_str!("../answers.txt");

#[derive(Parser, Debug)]
struct Args {
    #[clap(short, long)]
    implementation: Implementation,

    #[clap(short, long)]
    max: Option<usize>,
}

#[derive(ValueEnum, Debug, Clone, Copy)]
enum Implementation {
    Naive,
    Allocs,
    VecRem,
}

fn main() {
    let args = Args::parse();
    match args.implementation {
        Implementation::Naive => play(roget::algorithms::Naive::new, args.max),
        Implementation::Allocs => play(roget::algorithms::Allocs::new, args.max),
        Implementation::VecRem => play(roget::algorithms::VecRem::new, args.max),
    };
}

fn play<G: Guesser>(mut mk: impl FnMut() -> G, max: Option<usize>) {
    let w: roget::Wordle = roget::Wordle::new();
    for answer in GAMES.split_whitespace().take(max.unwrap_or(usize::MAX)) {
        let guesser = (mk)();
        if let Some(score) = w.play(answer, guesser) {
            println!("Guessed {answer} in {score}");
        } else {
            eprintln!("Failed to guess: {answer}");
        }
    }
}
