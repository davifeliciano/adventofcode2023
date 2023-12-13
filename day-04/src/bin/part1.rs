use std::{num::NonZeroUsize, thread};

use day_04::{cards_from_lines, ParseError};

fn main() -> Result<(), ParseError> {
    let lines: Vec<&str> = include_str!("../data/input.txt").lines().collect();
    let workers_count: usize = thread::available_parallelism()
        .unwrap_or(NonZeroUsize::new(2).unwrap())
        .into();

    let result: u32 = cards_from_lines(&lines, workers_count)?
        .iter()
        .map(|c| c.points())
        .sum();

    println!("Result: {}", result);

    Ok(())
}
