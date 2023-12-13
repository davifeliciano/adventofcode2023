use regex::{Match, Regex};
use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub struct BuildError(&'static str);

impl Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error building EngineSchematic: {}", self.0)
    }
}

impl Error for BuildError {}

fn get_enclosing_lines_indices(gear_line_index: usize, lines: usize) -> (usize, usize) {
    let start_line = match gear_line_index {
        line_index @ 0 => line_index,
        line_index @ _ => line_index - 1,
    };

    let end_line = if gear_line_index == lines - 1 {
        gear_line_index
    } else {
        gear_line_index + 1
    };

    (start_line, end_line)
}

fn get_match_boundary(re_match: Match<'_>, line_length: usize) -> (usize, usize) {
    let start = match re_match.start() {
        start @ 0 => start,
        start @ _ => start - 1,
    };

    let end = if re_match.end() == line_length {
        re_match.end()
    } else {
        re_match.end() + 1
    };

    (start, end)
}

fn indexes_distance(indexes: (usize, usize)) -> usize {
    if indexes.0 > indexes.1 {
        indexes.0 - indexes.1
    } else {
        indexes.1 - indexes.0
    }
}

#[derive(Debug)]
pub struct PartNumber<'a> {
    line_index: usize,
    line_length: usize,
    num_match: Match<'a>,
}

impl<'a> PartNumber<'a> {
    fn from_match(num_match: Match<'a>, line_index: usize, line_length: usize) -> Self {
        PartNumber {
            line_index,
            line_length,
            num_match,
        }
    }

    pub fn line_index(&self) -> usize {
        self.line_index
    }

    pub fn line_length(&self) -> usize {
        self.line_length
    }

    pub fn start(&self) -> usize {
        self.num_match.start()
    }

    pub fn end(&self) -> usize {
        self.num_match.end()
    }

    pub fn content(&self) -> &str {
        self.num_match.as_str()
    }

    fn has_gear_symbol(&self, gear_match: Match<'_>, gear_line_index: usize) -> bool {
        let (boundary_start, boundary_end) = get_match_boundary(self.num_match, self.line_length);

        indexes_distance((gear_line_index, self.line_index)) <= 1
            && boundary_start <= gear_match.start()
            && gear_match.end() <= boundary_end
    }
}

#[derive(Debug)]
pub struct EngineSchematic<'a> {
    lines: Vec<&'a str>,
    line_length: usize,
    part_numbers: Vec<Vec<PartNumber<'a>>>,
}

impl<'a> EngineSchematic<'a> {
    pub fn build(
        content: &'a str,
        part_number_pattern: &str,
        symbol_pattern: &str,
    ) -> Result<Self, BuildError> {
        let (line_length, lines) = Self::validate_content_lines(content)?;
        let part_number_regex = Regex::new(part_number_pattern)
            .map_err(|_| BuildError("invalid part_number_pattern"))?;

        let symbol_regex =
            Regex::new(symbol_pattern).map_err(|_| BuildError("invalid symbol_regex"))?;

        let mut schematic = EngineSchematic {
            lines,
            line_length,
            part_numbers: vec![],
        };

        schematic.set_part_numbers(&part_number_regex, &symbol_regex);

        Ok(schematic)
    }

    pub fn part_numbers(&self) -> &Vec<Vec<PartNumber<'_>>> {
        &self.part_numbers
    }

    fn validate_content_lines(content: &'a str) -> Result<(usize, Vec<&'a str>), BuildError> {
        let mut lines = content.lines();

        let line_length = lines.next().map_or_else(
            || Err(BuildError("input must not be empty")),
            |line| Ok(line.len()),
        )?;

        if !lines.all(|line| line.len() == line_length) {
            return Err(BuildError("lines in input does not have equal length"));
        }

        Ok((line_length, content.lines().collect()))
    }

    fn line_of_match_has_symbol(
        &self,
        symbol_regex: &Regex,
        line_index: usize,
        boundary: (usize, usize),
    ) -> bool {
        let line = self.lines[line_index];
        let (start, end) = boundary;
        let num_match_has_symbol_before = symbol_regex.is_match(&line[start..start + 1]);
        let num_match_has_symbol_after = symbol_regex.is_match(&line[end - 1..end]);

        num_match_has_symbol_before || num_match_has_symbol_after
    }

    fn line_before_match_has_symbol(
        &self,
        symbol_regex: &Regex,
        line_index: usize,
        boundary: (usize, usize),
    ) -> bool {
        line_index != 0
            && symbol_regex.is_match(&self.lines[line_index - 1][boundary.0..boundary.1])
    }

    fn line_after_match_has_symbol(
        &self,
        symbol_regex: &Regex,
        line_index: usize,
        boundary: (usize, usize),
    ) -> bool {
        line_index != self.lines.len() - 1
            && symbol_regex.is_match(&self.lines[line_index + 1][boundary.0..boundary.1])
    }

    fn match_is_part_number(
        &self,
        symbol_regex: &Regex,
        num_match: Match<'_>,
        line_index: usize,
    ) -> bool {
        let match_boundary = get_match_boundary(num_match, self.line_length);

        self.line_of_match_has_symbol(symbol_regex, line_index, match_boundary)
            || self.line_before_match_has_symbol(symbol_regex, line_index, match_boundary)
            || self.line_after_match_has_symbol(symbol_regex, line_index, match_boundary)
    }

    fn set_part_numbers(&mut self, part_number_regex: &Regex, symbol_regex: &Regex) {
        for line_index in 0..self.lines.len() {
            let mut line_part_numbers = vec![];

            for num_match in part_number_regex.find_iter(self.lines[line_index]) {
                if self.match_is_part_number(symbol_regex, num_match, line_index) {
                    line_part_numbers.push(PartNumber::from_match(
                        num_match,
                        line_index,
                        self.line_length,
                    ))
                }
            }

            self.part_numbers.push(line_part_numbers)
        }
    }

    fn get_number_pair_for_gear(
        &self,
        gear_match: Match<'_>,
        gear_line_index: usize,
    ) -> Option<[&PartNumber<'_>; 2]> {
        let (start_line, end_line) = get_enclosing_lines_indices(gear_line_index, self.lines.len());
        let mut gear_ratios = vec![];

        for line_part_numbers in &self.part_numbers[start_line..end_line + 1] {
            for part_number in line_part_numbers {
                if part_number.start() > gear_match.end() {
                    break;
                }

                let part_number_has_gear_symbol =
                    part_number.has_gear_symbol(gear_match, gear_line_index);

                if part_number_has_gear_symbol {
                    gear_ratios.push(part_number);
                }
            }
        }

        match gear_ratios.len() {
            2 => Some([gear_ratios[0], gear_ratios[1]]),
            _ => None,
        }
    }

    pub fn get_gear_ratios_pairs(
        &self,
        gear_symbol_regex: &Regex,
    ) -> Vec<Vec<[&PartNumber<'_>; 2]>> {
        let mut gear_ratios_pairs = vec![];

        for gear_line_index in 0..self.lines.len() {
            let mut line_gear_ratios = vec![];

            for gear_match in gear_symbol_regex.find_iter(self.lines[gear_line_index]) {
                if let Some(r) = self.get_number_pair_for_gear(gear_match, gear_line_index) {
                    line_gear_ratios.push(r);
                }
            }

            gear_ratios_pairs.push(line_gear_ratios);
        }

        gear_ratios_pairs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_numbers() {
        let content =
"............409..........784...578...802......64..............................486.248..............177....................369...............
.....-939..........524#...#....=.......*.........+......90.................................76..615..-..@.....961..........$.......*.........
............951*........................736...955..258....*.....253@.............210.10.....=...*.......776...*....&...............600..274.";

        let schematic = EngineSchematic::build(content, r"\d+", r"[^\.^\d]").unwrap();

        assert_eq!(
            schematic
                .part_numbers()
                .iter()
                .map(|v| v.iter().map(|n| n.content()).collect::<Vec<_>>())
                .collect::<Vec<_>>(),
            vec![
                vec!["784", "578", "802", "177", "369"],
                vec!["939", "524", "90", "76", "615", "961"],
                vec!["951", "736", "955", "253", "776", "600"]
            ]
        );
    }

    #[test]
    fn test_gear_ratios() {
        let content =
"..561..517..994*248.596......&...$.....196.701.....*............217...*....160........240..+....265..471..76............509..15........245..
.........*...............615.801.837......*......661.181...707......613...-.......495......959..........*....#...........*....$.............
.....-...107............@...............42.69.......*........*787................70*....$............853.....808..249.160.......725......151";

        let schematic = EngineSchematic::build(content, r"\d+", r"[^\.^\d]").unwrap();
        let gear_symbol_regex = Regex::new(r"\*").unwrap();

        assert_eq!(
            schematic
                .get_gear_ratios_pairs(&gear_symbol_regex)
                .iter()
                .flatten()
                .map(|a| a.iter().map(|n| n.content()).collect::<Vec<_>>())
                .collect::<Vec<_>>(),
            vec![
                vec!["994", "248"],
                vec!["517", "107"],
                vec!["471", "853"],
                vec!["509", "160"],
                vec!["661", "181"],
                vec!["707", "787"],
                vec!["495", "70"],
            ]
        )
    }
}
