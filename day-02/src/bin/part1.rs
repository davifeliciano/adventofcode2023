use day_02::{games_from_lines, CubeCollection, ParseError};
use std::{num::NonZeroUsize, str::FromStr, thread};

fn main() -> Result<(), ParseError> {
    let lines: Vec<&str> = include_str!("../data/input.txt").lines().collect();
    let workers_count: usize = thread::available_parallelism()
        .unwrap_or(NonZeroUsize::new(2).unwrap())
        .into();

    let cubes_in_bag = CubeCollection::from_str("12 red, 13 green, 14 blue").unwrap();
    let result: u32 = games_from_lines(&lines, workers_count)?
        .iter()
        .filter(|&game| game.is_possible(&cubes_in_bag))
        .map(|game| game.id)
        .sum();

    println!("Result: {}", result);

    Ok(())
}
