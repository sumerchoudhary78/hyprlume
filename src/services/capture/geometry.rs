use anyhow::{ensure, Context, Result};

pub(super) fn validate_geometry(value: &str) -> Result<()> {
    ensure!(value.len() <= 128, "Invalid region geometry");
    let parts: Vec<_> = value.split([',', ' ', 'x']).collect();
    ensure!(parts.len() == 4, "Invalid region geometry");
    let numbers: Vec<i32> = parts
        .into_iter()
        .map(str::parse)
        .collect::<std::result::Result<_, _>>()
        .context("Invalid region geometry")?;
    ensure!(
        numbers[0].abs_diff(0) <= 100_000
            && numbers[1].abs_diff(0) <= 100_000
            && (1..=32768).contains(&numbers[2])
            && (1..=32768).contains(&numbers[3]),
        "Invalid region dimensions"
    );
    Ok(())
}
