use day_02::{games_from_lines, ParseError};
use std::{num::NonZeroUsize, thread};

fn main() -> Result<(), ParseError> {
    let lines: Vec<&str> = include_str!("../data/input.txt").lines().collect();
    let workers_count: usize = thread::available_parallelism()
        .unwrap_or(NonZeroUsize::new(2).unwrap())
        .into();

    let result: u32 = games_from_lines(&lines, workers_count)?
        .iter()
        .map(|game| game.minimal_bag().power())
        .sum();

    println!("Result: {}", result);

    Ok(())
}
