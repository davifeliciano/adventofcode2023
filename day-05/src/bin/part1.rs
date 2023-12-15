use std::str::FromStr;

use day_05::{Almanac, Category, ParseError};

fn main() -> Result<(), ParseError> {
    let input = include_str!("../data/input.txt");
    let almanac = Almanac::from_str(input)?;
    let result = almanac
        .instructions()
        .iter()
        .map(|m| *m.get(&Category::Location).unwrap())
        .min()
        .unwrap();

    println!("Result: {}", result);

    Ok(())
}
