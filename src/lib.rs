use std::{collections::HashSet, str::FromStr};

#[derive(Debug)]
pub enum Error {
    InvalidRow(String),
}

pub struct ColumnParser<T> {
    columns: HashSet<u32>,
    delim: String,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: FromStr> ColumnParser<T> {
    pub fn new(columns: &[u32], delim: &str) -> Self {
        Self {
            columns: HashSet::from_iter(columns.iter().cloned()),
            delim: delim.to_string(),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn single(column: u32, delim: &str) -> Self {
        Self::new(&[column], delim)
    }

    pub fn parse_row(&self, row: &str) -> Result<Vec<T>, Error> {
        let mut result: Vec<T> = Vec::new();
        let vals: Vec<_> = row.split(&self.delim).collect();
        for column in &self.columns {
            let val = vals.get(*column as usize).ok_or(Error::InvalidRow(format!(
                "column {} does not exist in row",
                column
            )))?;

            let parsed = val
                .parse::<T>()
                .map_err(|_| Error::InvalidRow(format!("column {} cannot be parsed", column)))?;

            result.push(parsed);
        }

        Ok(result)
    }
}

pub struct Bin {
    pub label: f64,
    pub count: usize,
}

pub struct Histogram {
    bins: Vec<Bin>,
}

impl Histogram {
    pub fn from_values(values: Vec<f64>, num_bins: usize) -> Self {
        let bins = match values
            .iter()
            .fold(None, |acc: Option<(f64, f64)>, &value| match acc {
                Some((min, max)) => Some((min.min(value), max.max(value))),
                None => Some((f64::INFINITY, f64::NEG_INFINITY)),
            }) {
            Some((min, max)) => {
                let bin_width = (max - min) / num_bins as f64;
                let mut bins: Vec<Bin> = (0..num_bins)
                    .into_iter()
                    .map(|i| i as f64 * bin_width + min + bin_width / 2.0)
                    .map(|label| Bin { label, count: 0 })
                    .collect();

                values
                    .iter()
                    .map(|&value| {
                        // FIXME: Some kind of race condition here?
                        ((value - min) / (max.next_up() - min) * num_bins as f64).floor() as usize
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

    pub fn into_labels(self) -> Vec<f64> {
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

        let result: Vec<f64> = ColumnParser::<f64, _>::new(cursor, 1, ",")
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
