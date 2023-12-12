use regex::{Match, Matches, Regex};
use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub struct BuildError(&'static str);

impl Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error building EngineSchematic: {}", self.0)
    }
}

impl Error for BuildError {}

#[derive(Debug)]
pub struct EngineSchematic<'n, 's> {
    content: String,
    part_number_pattern: &'n str,
    part_number_regex: Regex,
    symbol_pattern: &'s str,
    symbol_regex: Regex,
}

impl<'n, 's> EngineSchematic<'n, 's> {
    pub fn build(
        content: &str,
        part_number_pattern: &'n str,
        symbol_pattern: &'s str,
    ) -> Result<Self, BuildError> {
        let mut lines = content.lines();
        let line_length = lines.next().map_or_else(
            || Err(BuildError("input must not be empty")),
            |line| Ok(line.len()),
        )?;

        if !lines.all(|line| line.len() == line_length) {
            return Err(BuildError("lines in input does not have equal length"));
        }

        let part_number_regex = Regex::new(part_number_pattern)
            .map_err(|_| BuildError("invalid part_number_pattern"))?;

        let symbol_regex =
            Regex::new(symbol_pattern).map_err(|_| BuildError("invalid symbol_regex"))?;

        Ok(EngineSchematic {
            content: content.to_string(),
            part_number_pattern,
            part_number_regex,
            symbol_pattern,
            symbol_regex,
        })
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn part_number_pattern(&self) -> &str {
        &self.part_number_pattern
    }

    pub fn symbol_pattern(&self) -> &str {
        &self.symbol_pattern
    }

    fn find_candidates_in_line(&'n self, line: &'n str) -> Matches<'_, '_> {
        self.part_number_regex.find_iter(line)
    }

    fn get_match_boundary(
        num_match: Match<'_>,
        lines: &[&str],
        line_index: usize,
    ) -> (usize, usize) {
        let line = lines[line_index];
        let start = match num_match.start() {
            start @ 0 => start,
            start @ _ => start - 1,
        };

        let end = if num_match.end() == line.len() {
            num_match.end()
        } else {
            num_match.end() + 1
        };

        (start, end)
    }

    fn line_of_match_has_symbol(
        &'n self,
        lines: &[&str],
        line_index: usize,
        boundary: (usize, usize),
    ) -> bool {
        let line = lines[line_index];
        let (start, end) = boundary;
        let num_match_has_symbol_before = self.symbol_regex.is_match(&line[start..start + 1]);
        let num_match_has_symbol_after = self.symbol_regex.is_match(&line[end - 1..end]);

        num_match_has_symbol_before || num_match_has_symbol_after
    }

    fn line_before_match_has_symbol(
        &'n self,
        lines: &[&str],
        line_index: usize,
        boundary: (usize, usize),
    ) -> bool {
        line_index != 0
            && self
                .symbol_regex
                .is_match(&lines[line_index - 1][boundary.0..boundary.1])
    }

    fn line_after_match_has_symbol(
        &'n self,
        lines: &[&str],
        line_index: usize,
        boundary: (usize, usize),
    ) -> bool {
        line_index != lines.len() - 1
            && self
                .symbol_regex
                .is_match(&lines[line_index + 1][boundary.0..boundary.1])
    }

    fn match_is_part_number(
        &'n self,
        num_match: Match<'_>,
        lines: &[&str],
        line_index: usize,
    ) -> bool {
        let match_boundary = Self::get_match_boundary(num_match, lines, line_index);

        self.line_of_match_has_symbol(lines, line_index, match_boundary)
            || self.line_before_match_has_symbol(lines, line_index, match_boundary)
            || self.line_after_match_has_symbol(lines, line_index, match_boundary)
    }

    pub fn part_numbers(&self) -> Vec<&str> {
        let lines: Vec<&str> = self.content.lines().collect();
        let mut result = vec![];

        for line_index in 0..lines.len() {
            for num_match in self.find_candidates_in_line(lines[line_index]) {
                if self.match_is_part_number(num_match, &lines, line_index) {
                    result.push(num_match.as_str())
                }
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_numbers() {
        let content = "............409..........784...578...802......64..............................486.248..............177....................369...............
.....-939..........524#...#....=.......*.........+......90.................................76..615..-..@.....961..........$.......*.........
............951*........................736...955..258....*.....253@.............210.10.....=...*.......776...*....&...............600..274.";

        let schematic = EngineSchematic::build(content, r"\d+", r"[^\.^\d]").unwrap();

        assert_eq!(
            schematic.part_numbers(),
            vec![
                "784", "578", "802", "177", "369", "939", "524", "90", "76", "615", "961", "951",
                "736", "955", "253", "776", "600"
            ]
        );
    }
}
