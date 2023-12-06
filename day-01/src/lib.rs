use std::{
    sync::{Arc, Mutex},
    thread,
};

pub fn concat_first_and_last_digits(line: &str) -> u32 {
    let mut digits = line.chars().filter(|c| c.is_numeric());
    let first = digits.next().unwrap_or('0');
    let last = digits.next_back().unwrap_or(first);
    let result = format!("{}{}", first, last);

    result.parse::<u32>().unwrap()
}

pub fn add_over_lines(
    lines: &[&str],
    worker_count: usize,
    adder: impl Fn(&str) -> u32 + Send + Copy,
) -> u32 {
    thread::scope(|s| {
        let chunks = lines.chunks(lines.len() / worker_count + 1);
        let sum = Arc::new(Mutex::new(0u32));
        let mut handles = vec![];

        for chunk in chunks {
            let sum = Arc::clone(&sum);
            handles.push(s.spawn(move || {
                for line in chunk {
                    let mut sum = sum.lock().unwrap();
                    *sum += adder(line);
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let result = *sum.lock().unwrap();
        result
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concat_first_and_last_digits() {
        assert_eq!(concat_first_and_last_digits("one"), 0);
        assert_eq!(concat_first_and_last_digits("one2three"), 22);
        assert_eq!(concat_first_and_last_digits("one2three4five"), 24);
        assert_eq!(concat_first_and_last_digits("one2three4five6seven"), 26);
    }

    #[test]
    fn test_add_over_lines() {
        let lines: Vec<&str> = "1abc2
pqr3stu8vwx
a1b2c3d4e5f
treb7uchet"
            .lines()
            .collect();

        assert_eq!(add_over_lines(&lines, 2, concat_first_and_last_digits), 142);
    }
}
