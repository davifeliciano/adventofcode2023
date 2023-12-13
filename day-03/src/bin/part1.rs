use day_03::{BuildError, EngineSchematic};

fn main() -> Result<(), BuildError> {
    let content = include_str!("../data/input.txt");
    let schematic = EngineSchematic::build(content, r"\d+", r"[^\.^\d]")?;

    let part_numbers_sum: u32 = schematic
        .part_numbers()
        .iter()
        .flatten()
        .map(|n| {
            n.content()
                .parse::<u32>()
                .expect("error parsing u32 from PartNumber contents")
        })
        .sum();

    println!("Result: {}", part_numbers_sum);

    Ok(())
}
