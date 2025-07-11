use std::io::{BufRead, Lines};
use std::str::FromStr;

#[derive(Debug)]
pub enum Error {
    InvalidRow(String),
    FailedRead(String),
}

// struct ColumnParser<T, Reader>
// - T must implement FromStr for ::parse<T>()
// - Reader must implement Read for reading rows
// - Takes:
//   - reader: Read (may be a Stdin or a BufReader<File>)
//   - column: usize
//   - delim: &str
// - Handle parsing into T using ::parse<T>()
// - Provides iterator over rows via .rows()
// - Columns are 0-indexed

pub struct ColumnParser<T, Reader> {
    lines: Lines<Reader>,
    column: usize,
    delim: String,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: FromStr, Reader: BufRead> ColumnParser<T, Reader> {
    pub fn new(reader: Reader, column: usize, delim: &str) -> Self {
        Self {
            lines: reader.lines(),
            column,
            delim: delim.to_string(),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn parse_row(&mut self) -> Option<Result<Option<T>, Error>> {
        // Get next line from BufRead.
        // If it returns None, there are no more lines to read.
        match self.lines.next()? {
            Ok(line) => {
                // Split the line by delim and return the nth column.
                match line.split(&self.delim).nth(self.column) {
                    Some(value) => Some(Ok(value.parse::<T>().ok())),

                    // Input is malformed or column is out of bounds.
                    None => Some(Err(Error::InvalidRow(format!(
                        "column {} does not exist in row",
                        self.column
                    )))),
                }
            }

            // If next() returns an Err, then reading input failed.
            Err(e) => Some(Err(Error::FailedRead(format!(
                "failed to read line: {}",
                e
            )))),
        }
    }

    pub fn rows(self) -> Rows<T, Reader> {
        Rows { parser: self }
    }
}

pub struct Rows<T, Reader> {
    parser: ColumnParser<T, Reader>,
}

impl<T: FromStr, Reader: BufRead> Iterator for Rows<T, Reader> {
    type Item = Result<Option<T>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.parser.parse_row()
    }
}

// struct Histogram<T>
// - Handles binning data on creation
//   - fn from_values(values: Vec<T>, num_bins: usize) -> Self
// - Can't be constructed from only an iterator
//   - Need max and min
//   - Need num_bins
// - Bins:
//   - Label
//   - Count
// - Can be converted to UTF-8:
//   - Into csv
//   - Into space delim
// - Can this be plotted?

pub struct Bin {
    pub label: f32,
    pub count: usize,
}

pub struct Histogram {
    bins: Vec<Bin>,
}

impl Histogram {
    pub fn from_values(values: Vec<f32>, num_bins: usize) -> Self {
        let bins = match values
            .iter()
            .fold(None, |acc: Option<(f32, f32)>, &value| match acc {
                Some((min, max)) => Some((min.min(value), max.max(value))),
                None => Some((f32::INFINITY, f32::NEG_INFINITY)),
            }) {
            Some((min, max)) => {
                let bin_width = (max - min) / num_bins as f32;
                let mut bins: Vec<Bin> = (0..num_bins)
                    .into_iter()
                    .map(|i| i as f32 * bin_width + min + bin_width / 2.0)
                    .map(|label| Bin { label, count: 0 })
                    .collect();

                values
                    .iter()
                    .map(|&value| {
                        ((value - min) / (max.next_up() - min) * num_bins as f32).trunc() as usize
                    })
                    .for_each(|i| bins[i].count += 1);

                bins
            }
            None => Vec::new(),
        };

        Histogram { bins }
    }

    pub fn into_bins(self) -> Vec<Bin> {
        self.bins
    }

    pub fn into_counts(self) -> Vec<usize> {
        self.bins.into_iter().map(|bin| bin.count).collect()
    }

    pub fn into_labels(self) -> Vec<f32> {
        self.bins.into_iter().map(|bin| bin.label).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_column_parser() {
        let data = "1.0,2.0,3.0\n4.0,5.0,6.0\n7.0,8.0,9.0\n";
        let cursor = Cursor::new(data);

        let result: Vec<f32> = ColumnParser::<f32, _>::new(cursor, 1, ",")
            .rows()
            .filter_map(|row| row.unwrap())
            .collect();

        assert_eq!(result, vec![2.0, 5.0, 8.0]);
    }

    #[test]
    fn test_histogram_counts_from_values() {
        let values = vec![2.0, 1.0, 2.0, 3.0, 3.0, 2.0, 0.0, 1.0, 1.0, 1.0];
        let histogram = Histogram::from_values(values, 3);
        assert_eq!(histogram.into_counts(), vec![5, 3, 2]);
    }

    #[test]
    fn test_histogram_labels_from_values() {
        let values = vec![2.0, 1.0, 2.0, 3.0, 3.0, 2.0, 0.0, 1.0, 1.0, 1.0];
        let histogram = Histogram::from_values(values, 3);
        assert_eq!(histogram.into_labels(), vec![0.5, 1.5, 2.5]);
    }
}
