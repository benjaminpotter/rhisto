use std::{collections::HashSet, str::FromStr};

#[derive(Debug, PartialEq)]
pub enum Error {
    MissingColumn(String, u32),
    FailedParse(String, String),
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
            let val = vals
                .get(*column as usize)
                .ok_or(Error::MissingColumn(row.to_string(), *column))?;

            let parsed = val.parse::<T>().map_err(|_| {
                Error::FailedParse(val.to_string(), std::any::type_name::<T>().to_string())
            })?;

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

    #[test]
    fn parse_column_from_rows() {
        let parser = ColumnParser::<f64>::single(1, ",");
        let result: Vec<f64> = vec!["1.0,2.0,3.0", "4.0,5.0,6.0", "7.0,8.0,9.0"]
            .into_iter()
            .map(|row| parser.parse_row(&row).unwrap()[0])
            .collect();

        assert_eq!(result, vec![2.0, 5.0, 8.0]);
    }

    #[test]
    fn missing_column() {
        let parser = ColumnParser::<f64>::new(&[1], ",");
        let result: Vec<Result<_, Error>> = vec!["1.0,2.0,3.0", "4.0", "7.0,8.0,9.0"]
            .into_iter()
            .map(|row| parser.parse_row(&row))
            .collect();

        assert_eq!(
            result,
            vec![
                Ok(vec![2.0]),
                Err(Error::MissingColumn("4.0".to_string(), 1)),
                Ok(vec![8.0])
            ]
        );
    }

    #[test]
    fn failed_parse() {
        let parser = ColumnParser::<f64>::new(&[1], ",");
        let result: Vec<Result<_, Error>> =
            vec!["1.0,2.0,3.0", "4.0,not_a_float,6.0", "7.0,8.0,9.0"]
                .into_iter()
                .map(|row| parser.parse_row(&row))
                .collect();

        assert_eq!(
            result,
            vec![
                Ok(vec![2.0]),
                Err(Error::FailedParse(
                    "not_a_float".to_string(),
                    "f64".to_string()
                )),
                Ok(vec![8.0])
            ]
        );
    }

    #[test]
    fn histogram_counts_from_values() {
        let values = vec![2.0, 1.0, 2.0, 3.0, 3.0, 2.0, 0.0, 1.0, 1.0, 1.0];
        let histogram = Histogram::from_values(values, 3);
        assert_eq!(histogram.into_counts(), vec![5, 3, 2]);
    }

    #[test]
    fn histogram_labels_from_values() {
        let values = vec![2.0, 1.0, 2.0, 3.0, 3.0, 2.0, 0.0, 1.0, 1.0, 1.0];
        let histogram = Histogram::from_values(values, 3);
        assert_eq!(histogram.into_labels(), vec![0.5, 1.5, 2.5]);
    }
}
