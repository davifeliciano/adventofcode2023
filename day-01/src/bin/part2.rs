use day_01::add_over_lines;
use std::{num::NonZeroUsize, thread};

const DIGITS_NAMES: [(&str, char); 10] = [
    ("zero", '0'),
    ("one", '1'),
    ("two", '2'),
    ("three", '3'),
    ("four", '4'),
    ("five", '5'),
    ("six", '6'),
    ("seven", '7'),
    ("eight", '8'),
    ("nine", '9'),
];

fn get_first_converted_digit(input: &str) -> Option<char> {
    for (i, c) in input.char_indices() {
        if c.is_numeric() {
            return Some(c);
        }

        for (digit_name, digit_char) in DIGITS_NAMES {
            let upper = (i + digit_name.len()).min(input.len());
            if digit_name == &input[i..upper] {
                return Some(digit_char);
            }
        }
    }

    None
}

fn get_last_converted_digit(input: &str) -> Option<char> {
    for (i, c) in input.char_indices().rev() {
        if c.is_numeric() {
            return Some(c);
        }

        for (digit_name, digit_char) in DIGITS_NAMES {
            let lower: usize = if digit_name.len() > i + 1 {
                continue;
            } else {
                i + 1 - digit_name.len()
            };

            if digit_name == &input[lower..=i] {
                return Some(digit_char);
            }
        }
    }

    None
}

fn concat_first_and_last_converted_digits(input: &str) -> u32 {
    let first = get_first_converted_digit(input).unwrap_or('0');
    let last = get_last_converted_digit(input).unwrap_or('0');
    format!("{}{}", first, last).parse::<u32>().unwrap()
}

fn main() {
    let lines: Vec<&str> = include_str!("../data/input.txt").lines().collect();
    let workers_count: usize = thread::available_parallelism()
        .unwrap_or(NonZeroUsize::new(2).unwrap())
        .into();

    let sum = add_over_lines(
        &lines,
        workers_count,
        concat_first_and_last_converted_digits,
    );

    println!("Result: {}", sum);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_first_converted_digit() {
        assert_eq!(get_first_converted_digit("asdfasdfasdf"), None);
        assert_eq!(get_first_converted_digit("two1nine"), Some('2'));
        assert_eq!(get_first_converted_digit("asdf1nine"), Some('1'));
        assert_eq!(get_first_converted_digit("asdfour"), Some('4'));
        assert_eq!(get_first_converted_digit("asdf5"), Some('5'));
        assert_eq!(get_first_converted_digit("7pqrstsixteen"), Some('7'));
    }

    #[test]
    fn test_get_last_converted_digit() {
        assert_eq!(get_last_converted_digit("asdfasdfasdf"), None);
        assert_eq!(get_last_converted_digit("two1nine"), Some('9'));
        assert_eq!(get_last_converted_digit("two1asdf"), Some('1'));
        assert_eq!(get_last_converted_digit("nineteen"), Some('9'));
        assert_eq!(get_last_converted_digit("5asdf"), Some('5'));
        assert_eq!(get_last_converted_digit("asdf5"), Some('5'));
    }

    #[test]
    fn test_concat_first_and_last_converted_digits() {
        assert_eq!(concat_first_and_last_converted_digits("asdfasdfasdf"), 0);
        assert_eq!(concat_first_and_last_converted_digits("two1nine"), 29);
        assert_eq!(concat_first_and_last_converted_digits("eightwothree"), 83);
        assert_eq!(concat_first_and_last_converted_digits("abone2threeyz"), 13);
        assert_eq!(concat_first_and_last_converted_digits("4asdfasdfasdf"), 44);
        assert_eq!(concat_first_and_last_converted_digits("asdfasdfasdf1"), 11);
        assert_eq!(concat_first_and_last_converted_digits("twoasdfasdf"), 22);
        assert_eq!(concat_first_and_last_converted_digits("asdfasdftwo"), 22);
        assert_eq!(concat_first_and_last_converted_digits("7pqrstsixteen"), 76);
    }
}
