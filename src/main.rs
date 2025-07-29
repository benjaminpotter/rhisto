use clap::Parser;
use meval::Context;
use regex::Regex;
use rhisto::{ColumnParser, Histogram};
use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::PathBuf,
};

fn main() {
    let args = Args::parse();

    let mut reader: Box<dyn BufRead> = match args.input {
        Some(path_buf) => Box::new(BufReader::new(
            File::open(&path_buf).expect("failed to open input file"),
        )),
        None => Box::new(BufReader::new(std::io::stdin())),
    };

    if args.skip_header {
        reader.skip_until(b'\n').expect("failed to skip header");
    }

    let values: Vec<f64> = match args.column {
        Some(column) => {
            let parser = ColumnParser::<f64>::single(column, &args.delim);
            reader
                .lines()
                .map(|row| row.unwrap())
                .map(|row| parser.parse_row(&row).unwrap()[0])
                .collect()
        }
        None => {
            let expr = args.expr.unwrap();
            let re = Regex::new(r"\?([0-9]*)").unwrap();
            let columns: Vec<u32> = re
                .captures_iter(&expr)
                .map(|c| c.extract())
                .map(|(_, [col])| col.parse::<u32>().unwrap())
                .collect();

            let expr_repl = expr.replace("?", "_");
            let vars: Vec<String> = columns.iter().map(|col| format!("_{}", col)).collect();

            let parser = ColumnParser::<f64>::new(&columns[..], &args.delim);
            reader
                .lines()
                .map(|row| row.unwrap())
                .map(|row| parser.parse_row(&row).unwrap())
                .map(|vals| {
                    let mut ctx = Context::new();
                    for (var, val) in vars.iter().zip(vals.into_iter()) {
                        ctx.var(var, val);
                    }

                    meval::eval_str_with_context(&expr_repl, &ctx).unwrap()
                })
                .collect()
        }
    };

    let histo = Histogram::from_values(values, args.num_bins);

    let mut writer: Box<dyn Write> = match args.output {
        Some(path_buf) => Box::new(BufWriter::new(
            File::create(&path_buf).expect("failed to open output file"),
        )),
        None => Box::new(BufWriter::new(std::io::stdout())),
    };

    for bin in histo.into_bins().iter() {
        let _ = writeln!(writer, "{:0.2}{}{:0.2}", bin.label, &args.delim, bin.count);
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The optional buffer to read data from.
    input: Option<PathBuf>,

    /// The optional path to write histogram data to.
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// The zero indexed column in the input buffer to read.
    #[arg(short, long, group = "value")]
    column: Option<u32>,

    /// The expression over column indices used to compute histogram values.
    ///
    /// The `?` prefixes a column index in an expression.
    /// The expression is evaluated on each row.
    /// Any `?`ed column indices are bound to a concrete value for each row.
    #[arg(short, long, group = "value")]
    expr: Option<String>,

    /// The delimeting pattern used to separate columns in the input.
    #[arg(short, long, default_value = ",")]
    delim: String,

    /// Indicate whether the input data contains a header row.
    #[arg(short, long, default_value_t = false)]
    skip_header: bool,

    /// The number of bins in the histogram.
    #[arg(short, long, default_value_t = 10)]
    num_bins: usize,
}
