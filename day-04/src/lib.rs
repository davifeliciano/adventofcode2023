use std::{collections::HashSet, error::Error, fmt::Display, str::FromStr, thread};

#[derive(Debug)]
pub struct ParseError(String);

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error parsing card from string: {}", self.0)
    }
}

impl Error for ParseError {}

pub struct Card {
    pub id: usize,
    pub winning_numbers: HashSet<u32>,
    pub numbers: HashSet<u32>,
    copies: usize,
}

impl FromStr for Card {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let frags = &s.split(":").collect::<Vec<_>>()[..];

        let (card_id_str, numbers_str) = match frags {
            [card_id_str, numbers_str] => Ok((card_id_str, numbers_str)),
            _ => Err(ParseError(s.to_string())),
        }?;

        let id = Self::parse_card_id(card_id_str)?;
        let frags = &numbers_str.split("|").collect::<Vec<_>>()[..];

        let (winning_numbers_str, numbers_str) = match frags {
            [winning_numbers_str, numbers_str] => Ok((winning_numbers_str, numbers_str)),
            _ => Err(ParseError(s.to_string())),
        }?;

        let winning_numbers = Self::parse_numbers(winning_numbers_str)?;
        let numbers = Self::parse_numbers(numbers_str)?;

        Ok(Card {
            id,
            winning_numbers,
            numbers,
            copies: 1,
        })
    }
}

impl Card {
    fn parse_card_id(s: &str) -> Result<usize, ParseError> {
        match &s.split(" ").filter(|s| !s.is_empty()).collect::<Vec<_>>()[..] {
            ["Card", id] => id.parse::<usize>().map_err(|_| ParseError(s.to_string())),
            _ => Err(ParseError(s.to_string())),
        }
    }

    fn parse_numbers(s: &str) -> Result<HashSet<u32>, ParseError> {
        Ok(s.split(" ")
            .filter(|&s| !s.is_empty())
            .map(|s| s.parse::<u32>().map_err(|_| ParseError(s.to_string())))
            .collect::<Result<HashSet<_>, _>>()?)
    }

    pub fn hits_count(&self) -> usize {
        self.winning_numbers.intersection(&self.numbers).count()
    }

    pub fn points(&self) -> u32 {
        let hits_count = self.hits_count();

        if hits_count == 0 {
            0
        } else {
            2u32.pow(self.hits_count() as u32 - 1)
        }
    }

    pub fn copies(&self) -> usize {
        self.copies
    }

    pub fn copy(&mut self) {
        self.copies += 1
    }
}

pub struct CardCollection {
    cards: Vec<Card>,
}

impl CardCollection {
    pub fn from_lines(lines: &[&str], workers_count: usize) -> Result<CardCollection, ParseError> {
        let cards = cards_from_lines(lines, workers_count)?;
        Ok(CardCollection { cards }.increment_copies())
    }

    fn increment_copies(mut self) -> Self {
        let cards_count = self.cards.len();

        for i in 0..cards_count - 1 {
            let card = &self.cards[i];
            let hits_count = card.hits_count();
            let start = cards_count.min(i + 1);
            let end = cards_count.min(start + hits_count);

            for _ in 0..card.copies() {
                for card in &mut self.cards[start..end] {
                    card.copy();
                }
            }
        }

        self
    }

    pub fn cards(&self) -> &Vec<Card> {
        &self.cards
    }

    pub fn total_points(&self) -> u32 {
        self.cards.iter().map(|c| c.points()).sum()
    }

    pub fn total_copies(&self) -> usize {
        self.cards.iter().map(|c| c.copies()).sum()
    }
}

fn cards_from_lines(lines: &[&str], workers_count: usize) -> Result<Vec<Card>, ParseError> {
    thread::scope(|s| {
        let chunks = lines.chunks(lines.len() / workers_count + 1);
        let mut handles = vec![];
        let mut cards = vec![];

        for chunk in chunks {
            handles.push(s.spawn(move || {
                chunk
                    .iter()
                    .map(|line| Card::from_str(&line))
                    .collect::<Result<Vec<Card>, ParseError>>()
            }));
        }

        for handle in handles {
            let mut chunk_cards = handle.join().unwrap()?;
            cards.append(&mut chunk_cards);
        }

        Ok(cards)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_points() {
        let lines: Vec<_> = "Card 1: 41 48 83 86 17 | 83 86  6 31 17  9 48 53
Card 2: 13 32 20 16 61 | 61 30 68 82 17 32 24 19
Card 3:  1 21 53 59 44 | 69 82 63 72 16 21 14  1
Card 4: 41 92 73 84 69 | 59 84 76 51 58  5 54 83
Card 5: 87 83 26 28 32 | 88 30 70 12 93 22 82 36
Card 6: 31 18 13 56 72 | 74 77 10 23 35 67 36 11"
            .lines()
            .collect();

        let points: Vec<u32> = CardCollection::from_lines(&lines, 2)
            .unwrap()
            .cards()
            .iter()
            .map(|c| c.points())
            .collect();

        assert_eq!(points, vec![8, 2, 2, 1, 0, 0])
    }

    #[test]
    fn test_copies() {
        let lines: Vec<_> = "Card 1: 41 48 83 86 17 | 83 86  6 31 17  9 48 53
Card 2: 13 32 20 16 61 | 61 30 68 82 17 32 24 19
Card 3:  1 21 53 59 44 | 69 82 63 72 16 21 14  1
Card 4: 41 92 73 84 69 | 59 84 76 51 58  5 54 83
Card 5: 87 83 26 28 32 | 88 30 70 12 93 22 82 36
Card 6: 31 18 13 56 72 | 74 77 10 23 35 67 36 11"
            .lines()
            .collect();

        let copies: Vec<usize> = CardCollection::from_lines(&lines, 2)
            .unwrap()
            .cards()
            .iter()
            .map(|c| c.copies())
            .collect();

        assert_eq!(copies, vec![1, 2, 4, 8, 14, 1])
    }
}
