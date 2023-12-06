use day_01::add_over_lines;
use std::{num::NonZeroUsize, thread};

fn concat_first_and_last_digits(line: &str) -> u32 {
    let mut digits = line.chars().filter(|c| c.is_numeric());
    let first = digits.next().unwrap_or('0');
    let last = digits.next_back().unwrap_or(first);
    let result = format!("{}{}", first, last);

    result.parse::<u32>().unwrap()
}

fn main() {
    let lines: Vec<&str> = include_str!("../data/input.txt").lines().collect();
    let workers_count: usize = thread::available_parallelism()
        .unwrap_or(NonZeroUsize::new(2).unwrap())
        .into();

    let sum = add_over_lines(&lines, workers_count, concat_first_and_last_digits);
    println!("Result: {}", &sum);
}
