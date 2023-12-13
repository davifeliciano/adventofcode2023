use std::{num::NonZeroUsize, thread};

use day_04::{CardCollection, ParseError};

fn main() -> Result<(), ParseError> {
    let lines: Vec<&str> = include_str!("../data/input.txt").lines().collect();
    let workers_count: usize = thread::available_parallelism()
        .unwrap_or(NonZeroUsize::new(2).unwrap())
        .into();

    let result = CardCollection::from_lines(&lines, workers_count)?.total_copies();

    println!("Result: {}", result);

    Ok(())
}
