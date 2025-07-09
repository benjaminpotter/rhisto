#[derive(Debug)]
pub enum Error {
    InvalidRow(String),
    InvalidColumn(String),
}

/// Returns the index of a bin for a data point `x` between `min` and `max` inclusive.
pub fn get_bin_index(x: f32, min: f32, max: f32, bins: usize) -> usize {
    ((x - min) / (max.next_up() - min) * bins as f32).trunc() as usize
}

/// Returns the midpoint of the bin interval for a given index.
pub fn get_bin_label(index: usize, min: f32, max: f32, bins: usize) -> f32 {
    let bin_width = (max - min) / bins as f32;
    index as f32 * bin_width + min + bin_width / 2.0
}

/// Return the f32 at `column` delimited by `delim` in `buffer`.
pub fn get_data_at_column(buffer: &String, column: usize, delim: &str) -> Result<f32, Error> {
    let col_index = column.checked_sub(1).ok_or(Error::InvalidColumn(format!(
        "first column index is 1 but got {}",
        column
    )))?;

    let data = buffer
        .split(delim)
        .nth(col_index)
        .ok_or(Error::InvalidColumn(format!(
            "column {} out of bounds",
            column
        )))?;

    data.parse()
        .map_err(|e| Error::InvalidRow(format!("failed to parse f32 from {}: {}", data, e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_bin_index() {
        assert_eq!(get_bin_index(0.0, 0.0, 10.0, 10), 0);
        assert_eq!(get_bin_index(9.9, 0.0, 10.0, 10), 9);
        assert_eq!(get_bin_index(5.0, 0.0, 10.0, 10), 4);

        assert_eq!(get_bin_index(-1.0, -1.0, 1.0, 20), 0);
        assert_eq!(get_bin_index(0.9, -1.0, 1.0, 20), 19);
    }

    #[test]
    fn test_get_bin_label() {
        assert_eq!(get_bin_label(0, 0.0, 10.0, 10), 0.5);
        assert_eq!(get_bin_label(9, 0.0, 10.0, 10), 9.5);
        assert_eq!(get_bin_label(5, 0.0, 10.0, 10), 5.5);

        assert_eq!(get_bin_label(0, -1.0, 1.0, 20), -0.95);
        assert_eq!(get_bin_label(19, -1.0, 1.0, 20), 0.95);
    }
}
