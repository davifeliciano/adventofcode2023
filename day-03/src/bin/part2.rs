use day_03::{BuildError, EngineSchematic};
use regex::Regex;

fn main() -> Result<(), BuildError> {
    let content = include_str!("../data/input.txt");
    let schematic = EngineSchematic::build(content, r"\d+", r"[^\.^\d]")?;
    let gear_symbol_regex = Regex::new(r"\*").unwrap();

    let gear_ratios_sum: u32 = schematic
        .get_gear_ratios_pairs(&gear_symbol_regex)
        .iter()
        .flatten()
        .map(|a| {
            a.iter()
                .map(|n| {
                    n.content()
                        .parse::<u32>()
                        .expect("error parsing u32 from PartNumber contents")
                })
                .product::<u32>()
        })
        .sum::<u32>();

    println!("Result: {}", gear_ratios_sum);

    Ok(())
}
