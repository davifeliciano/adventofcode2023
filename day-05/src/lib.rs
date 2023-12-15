use std::{collections::HashMap, error::Error, fmt::Display, str::FromStr};

#[derive(Debug)]
pub struct ParseError(String);

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error parsing data from string: {}", self.0)
    }
}

impl Error for ParseError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Category {
    Seed,
    Soil,
    Fertilizer,
    Water,
    Light,
    Temperature,
    Humidity,
    Location,
}

impl FromStr for Category {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "seed" => Ok(Self::Seed),
            "soil" => Ok(Self::Soil),
            "fertilizer" => Ok(Self::Fertilizer),
            "water" => Ok(Self::Water),
            "light" => Ok(Self::Light),
            "temperature" => Ok(Self::Temperature),
            "humidity" => Ok(Self::Humidity),
            "location" => Ok(Self::Location),
            _ => Err(ParseError(format!("invalid category `{}`", s))),
        }
    }
}

impl Category {
    fn next(&self) -> Option<Self> {
        match self {
            Self::Seed => Some(Self::Soil),
            Self::Soil => Some(Self::Fertilizer),
            Self::Fertilizer => Some(Self::Water),
            Self::Water => Some(Self::Light),
            Self::Light => Some(Self::Temperature),
            Self::Temperature => Some(Self::Humidity),
            Self::Humidity => Some(Self::Location),
            Self::Location => None,
        }
    }
}

#[derive(Debug)]
struct Range {
    destination_start: u32,
    source_start: u32,
    length: u32,
}

impl FromStr for Range {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let range_ints = s
            .split(" ")
            .filter(|s| !s.is_empty())
            .map(|s| {
                s.parse::<u32>()
                    .map_err(|_| ParseError(format!("invalid int `{}` in CategoryMap ranges", s)))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let (destination_start, source_start, length) = match &range_ints[..] {
            [dest_start, source_start, length] => Ok((*dest_start, *source_start, *length)),
            _ => Err(ParseError(format!("invalid range `{}` in CategoryMap", s))),
        }?;

        Ok(Self {
            destination_start,
            source_start,
            length,
        })
    }
}

impl Range {
    fn get_destination(&self, source: u32) -> Option<u32> {
        let source_end = self.source_start + self.length;

        if source >= self.source_start && source < source_end {
            let offset = source - self.source_start;
            Some(self.destination_start + offset)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct CategoryMap {
    source: Category,
    ranges: Vec<Range>,
}

impl FromStr for CategoryMap {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();
        let header_line = lines.next().ok_or(ParseError(
            "missing source and destination in CategoryMap".to_string(),
        ))?;

        let header_tokens = header_line.split(" ").collect::<Vec<_>>();

        let src_and_dest = match &header_tokens[..] {
            [src_and_dest, "map:"] => Ok(src_and_dest),
            _ => Err(ParseError(format!(
                "invalid header `{}` in CategoryMap",
                header_line
            ))),
        }?;

        let source = match &src_and_dest.split("-to-").collect::<Vec<_>>()[..] {
            [src, dest] => {
                let source = Category::from_str(src)?;
                let destination = Category::from_str(dest)?;

                match source.next() {
                    Some(n) if n == destination => Ok(()),
                    _ => Err(ParseError(format!("forbidden map `{}`", src_and_dest))),
                }?;

                Ok(source)
            }
            _ => Err(ParseError(format!(
                "invalid header `{}` in CategoryMap",
                header_line
            ))),
        }?;

        let ranges = lines
            .map(|s| Range::from_str(s))
            .collect::<Result<Vec<_>, _>>()?;

        if ranges.is_empty() {
            return Err(ParseError(format!(
                "missing ranges in `{}` map in Almanac",
                src_and_dest
            )));
        }

        Ok(CategoryMap { source, ranges })
    }
}

impl CategoryMap {
    pub fn get_destination(&self, source: u32) -> u32 {
        self.ranges
            .iter()
            .find_map(|r| r.get_destination(source))
            .unwrap_or(source)
    }
}

#[derive(Debug)]
pub struct Almanac {
    seeds: Vec<u32>,
    category_maps: HashMap<Category, CategoryMap>,
}

impl FromStr for Almanac {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut frags = s.split("\n\n");
        let seeds_str = frags.next().ok_or(ParseError(
            "could not parse Almanac from empty string".to_string(),
        ))?;

        let seeds = match &seeds_str.split(": ").collect::<Vec<_>>()[..] {
            ["seeds", seeds_str] => {
                let seeds = seeds_str
                    .split(" ")
                    .filter(|s| !s.is_empty())
                    .map(|s| {
                        s.parse::<u32>().map_err(|_| {
                            ParseError(format!("invalid int `{}` in Almanac seeds", s))
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(seeds)
            }
            _ => Err(ParseError(format!(
                "invalid header `{}` in Almanac",
                seeds_str
            ))),
        }?;

        if seeds.is_empty() {
            return Err(ParseError("missing seeds in Almanac".to_string()));
        }

        let category_maps = frags
            .map(|s| {
                let category_map = CategoryMap::from_str(s)?;
                let source = category_map.source;

                Ok((source, category_map))
            })
            .collect::<Result<HashMap<_, _>, _>>()?;

        Ok(Almanac {
            seeds,
            category_maps,
        })
    }
}

impl Almanac {
    pub fn instructions(&self) -> Vec<HashMap<Category, u32>> {
        self.seeds
            .iter()
            .map(|seed| {
                let mut instructions = HashMap::new();
                let mut cur_cat = Category::Seed;
                let mut cur_number = *seed;

                loop {
                    instructions.insert(cur_cat, cur_number);

                    let cat_map = if let Some(cat_map) = self.category_maps.get(&cur_cat) {
                        cat_map
                    } else {
                        break;
                    };

                    cur_number = cat_map.get_destination(cur_number);
                    cur_cat = cur_cat.next().unwrap();
                }

                instructions
            })
            .collect::<Vec<_>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_locations() {
        let input = "seeds: 79 14 55 13

seed-to-soil map:
50 98 2
52 50 48

soil-to-fertilizer map:
0 15 37
37 52 2
39 0 15

fertilizer-to-water map:
49 53 8
0 11 42
42 0 7
57 7 4

water-to-light map:
88 18 7
18 25 70

light-to-temperature map:
45 77 23
81 45 19
68 64 13

temperature-to-humidity map:
0 69 1
1 0 69

humidity-to-location map:
60 56 37
56 93 4";

        let almanac = Almanac::from_str(input).unwrap();
        let locations: Vec<u32> = almanac
            .instructions()
            .iter()
            .map(|m| *m.get(&Category::Location).unwrap())
            .collect();

        assert_eq!(locations, vec![82, 43, 86, 35]);
    }
}
