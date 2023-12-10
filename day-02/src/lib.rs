use std::{collections::HashMap, error::Error, fmt::Display, str::FromStr, thread};

#[derive(Debug)]
pub struct ParseError(&'static str);

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error parsing game string: {}", self.0)
    }
}

impl Error for ParseError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Cube {
    Red,
    Green,
    Blue,
}

impl FromStr for Cube {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "red" => Ok(Self::Red),
            "green" => Ok(Self::Green),
            "blue" => Ok(Self::Blue),
            _ => Err(ParseError("invalid cube color")),
        }
    }
}

#[derive(Debug, Default)]
pub struct CubeCollection {
    pub stats: HashMap<Cube, u32>,
}

impl CubeCollection {
    pub fn contains(&self, other: &Self) -> bool {
        for (cube, count) in &self.stats {
            let other_count = *other.stats.get(cube).unwrap_or(&0);

            if other_count > *count {
                return false;
            }
        }

        true
    }

    pub fn power(&self) -> u32 {
        self.stats.iter().map(|(_, &count)| count).product()
    }
}

impl FromStr for CubeCollection {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut stats = HashMap::new();

        for s in s.split(", ") {
            match &s.split(' ').collect::<Vec<_>>()[..] {
                [count_str, color] => {
                    let cube = Cube::from_str(&color)?;
                    let count = count_str
                        .parse::<u32>()
                        .map_err(|_| ParseError("invalid cube count"))?;

                    if let Some(_) = stats.get(&cube) {
                        return Err(ParseError("repeated color in draw stats"));
                    }

                    stats.insert(cube, count);
                }
                _ => return Err(ParseError("invalid game structure")),
            }
        }

        Ok(CubeCollection { stats })
    }
}

#[derive(Debug)]
pub struct Game {
    pub id: u32,
    pub draws: Vec<CubeCollection>,
}

impl Game {
    pub fn is_possible(&self, cubes_in_bag: &CubeCollection) -> bool {
        self.draws.iter().all(|c| cubes_in_bag.contains(c))
    }

    pub fn minimal_bag(&self) -> CubeCollection {
        let mut result = CubeCollection::default();

        for draw in &self.draws {
            for (cube, count) in &draw.stats {
                let cur_max = result.stats.entry(*cube).or_default();

                if count > cur_max {
                    *cur_max = *count;
                }
            }
        }

        result
    }
}

fn parse_id(s: &str) -> Result<u32, ParseError> {
    match &s.split(' ').collect::<Vec<_>>()[..] {
        ["Game", id] => id.parse::<u32>().map_err(|_| ParseError("invalid game id")),
        _ => Err(ParseError("invalid game structure")),
    }
}

fn parse_draws(s: &str) -> Result<Vec<CubeCollection>, ParseError> {
    let mut draws = vec![];

    for s in s.split("; ") {
        let cube_collection = CubeCollection::from_str(s)?;
        draws.push(cube_collection);
    }

    Ok(draws)
}

impl FromStr for Game {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &s.split(": ").collect::<Vec<_>>()[..] {
            [id_str, draws_str] => {
                let id = parse_id(&id_str)?;
                let draws = parse_draws(&draws_str)?;
                Ok(Self { id, draws })
            }
            _ => Err(ParseError("invalid game structure")),
        }
    }
}

pub fn games_from_lines(lines: &[&str], worker_count: usize) -> Result<Vec<Game>, ParseError> {
    thread::scope(|s| {
        let chunks = lines.chunks(lines.len() / worker_count + 1);
        let mut handles = vec![];
        let mut games = vec![];

        for chunk in chunks {
            handles.push(s.spawn(move || {
                chunk
                    .iter()
                    .map(|line| Game::from_str(&line))
                    .collect::<Result<Vec<Game>, ParseError>>()
            }));
        }

        for handle in handles {
            let mut chunk_game = handle.join().unwrap()?;
            games.append(&mut chunk_game);
        }

        Ok(games)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_parser_valid() {
        let game_str = "Game 1: 3 blue, 4 red; 1 red, 2 green, 6 blue; 2 green";
        let game = Game::from_str(game_str).unwrap();
        assert_eq!(game.id, 1);
        assert_eq!(game.draws.len(), 3);
        assert_eq!(*game.draws[0].stats.get(&Cube::Blue).unwrap(), 3);
        assert_eq!(*game.draws[0].stats.get(&Cube::Red).unwrap(), 4);
        assert_eq!(*game.draws[1].stats.get(&Cube::Red).unwrap(), 1);
        assert_eq!(*game.draws[1].stats.get(&Cube::Green).unwrap(), 2);
        assert_eq!(*game.draws[1].stats.get(&Cube::Blue).unwrap(), 6);
        assert_eq!(*game.draws[2].stats.get(&Cube::Green).unwrap(), 2);
    }

    #[test]
    fn test_game_parser_invalid() {
        assert!(Game::from_str("asdf 1: 3 blue, 4 red; 1 red, 2 green").is_err());
        assert!(Game::from_str("Game X: 3 blue, 4 red; 1 red, 2 green").is_err());
        assert!(Game::from_str("Game 1| 3 blue, 4 red; 1 red, 2 green").is_err());
        assert!(Game::from_str("Game 1: X blue, 4 red; 1 red, 2 green").is_err());
        assert!(Game::from_str("Game 1: 3 asdf, 4 red; 1 red, 2 green").is_err());
        assert!(Game::from_str("Game 1: 3 blue, 4 red| 1 red, 2 green").is_err());
    }

    #[test]
    fn test_cube_collection_contains() {
        assert!(CubeCollection::from_str("1 blue")
            .unwrap()
            .contains(&CubeCollection::from_str("1 blue").unwrap()));

        assert!(CubeCollection::from_str("1 blue")
            .unwrap()
            .contains(&CubeCollection::from_str("1 red").unwrap()));

        assert!(CubeCollection::from_str("2 blue")
            .unwrap()
            .contains(&CubeCollection::from_str("1 blue").unwrap()));

        assert!(!CubeCollection::from_str("1 blue")
            .unwrap()
            .contains(&CubeCollection::from_str("2 blue").unwrap()));
    }

    #[test]
    fn test_cube_collection_power() {
        assert_eq!(CubeCollection::default().power(), 1);
        assert_eq!(CubeCollection::from_str("2 red").unwrap().power(), 2);
        assert_eq!(
            CubeCollection::from_str("2 red, 3 blue").unwrap().power(),
            6
        );
    }

    #[test]
    fn test_game_is_possible() {
        let lines: Vec<&str> = "Game 1: 3 blue, 4 red; 1 red, 2 green, 6 blue; 2 green
Game 2: 1 blue, 2 green; 3 green, 4 blue, 1 red; 1 green, 1 blue
Game 3: 8 green, 6 blue, 20 red; 5 blue, 4 red, 13 green; 5 green, 1 red
Game 4: 1 green, 3 red, 6 blue; 3 green, 6 red; 3 green, 15 blue, 14 red
Game 5: 6 red, 1 blue, 3 green; 2 blue, 1 red, 2 green"
            .lines()
            .collect();

        let cubes_in_bag = CubeCollection::from_str("12 red, 13 green, 14 blue").unwrap();
        let games = games_from_lines(&lines, 2).unwrap();
        let mut possible_games_ids = vec![];
        let mut impossible_games_ids = vec![];

        for game in games {
            if game.is_possible(&cubes_in_bag) {
                possible_games_ids.push(game.id);
            } else {
                impossible_games_ids.push(game.id);
            }
        }

        assert_eq!(possible_games_ids, vec![1, 2, 5]);
        assert_eq!(impossible_games_ids, vec![3, 4]);
    }

    #[test]
    fn test_game_minimal_bag() {
        let lines: Vec<&str> = "Game 1: 3 blue, 4 red; 1 red, 2 green, 6 blue; 2 green
Game 2: 1 blue, 2 green; 3 green, 4 blue, 1 red; 1 green, 1 blue
Game 3: 8 green, 6 blue, 20 red; 5 blue, 4 red, 13 green; 5 green, 1 red
Game 4: 1 green, 3 red, 6 blue; 3 green, 6 red; 3 green, 15 blue, 14 red
Game 5: 6 red, 1 blue, 3 green; 2 blue, 1 red, 2 green"
            .lines()
            .collect();

        let powers: Vec<u32> = games_from_lines(&lines, 2)
            .unwrap()
            .iter()
            .map(|game| game.minimal_bag().power())
            .collect();

        assert_eq!(powers, vec![48, 12, 1560, 630, 36]);
    }
}
